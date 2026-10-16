// 端到端测试：命运编织系统完整业务流程（task 19.1-19.6）
//
// 通过真实 CLI 子进程模拟 Spec 运行层视角的完整创作流程：
// 1. 初始化项目 → 创建卷 → 创建章节 → 创建主线节点 → 标记 achieved
// 2. 创建线索 → 绑定到多个章节（mention/develop/resolve）→ 查询章节线索
// 3. 创建涌现 → 评分 → 决策 accept → 执行到章节 → 归档（完整流程）
// 4. 创建涌现 → 评分 → 决策 defer → 验证无法执行
// 5. 创建涌现 → 评分 → 决策 reject → 验证无法执行

#[path = "../integration/common.rs"]
mod common;

use serde_json::Value;

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap()
}

#[test]
fn e2e_mainplot_full_flow() {
    // 初始化项目 → 创建卷 → 创建章节 → 创建主线节点 → 标记 achieved
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // 创建主线节点关联该章节
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "主线锚点",
        "--description", "卷一关键转折",
        "--target-chapters", &ch_id,
        "--json",
    ]);
    let node = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(node["title"], "主线锚点");
    assert_eq!(node["achieved"], 0);
    let targets = node["targetChapters"].as_array().unwrap();
    assert_eq!(targets[0], ch_id);

    // 标记达成
    let node_id = node["id"].as_str().unwrap().to_string();
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "update", &node_id, "--achieved", "1"]);
    cmd.assert().success();

    // 验证状态
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "get", &node_id, "--json"]);
    let node = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(node["achieved"], 1);

    // 列举卷内节点
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "list", &vid.to_string(), "--json"]);
    let list = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(list["nodes"].as_array().unwrap().len(), 1);
}

#[test]
fn e2e_thread_multi_chapter_bindings() {
    // 创建线索 → 绑定到多个章节（mention/develop/resolve）→ 查询
    let (dir, vid, ch1) = common::setup_world_with_volume_and_chapter();

    // 创建第二个章节
    let mut cmd = common::novel_at(&dir);
    cmd.args(["chapter", "create", "第二章", "--json"]);
    let ch2 = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 创建线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "复仇线", "--description", "主角复仇", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 绑定到 ch1（mention）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch1, "--type", "mention", "--note", "首次提及"]);
    cmd.assert().success();

    // 绑定到 ch2（develop）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch2, "--type", "develop", "--note", "发展推进"]);
    cmd.assert().success();

    // 查询 ch1 的线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "query", "--chapter", &ch1, "--json"]);
    let bindings = parse_json(&cmd.assert().success().get_output().stdout)["bindings"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0]["bindingType"], "mention");

    // 查询 ch2 的线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "query", "--chapter", &ch2, "--json"]);
    let bindings = parse_json(&cmd.assert().success().get_output().stdout)["bindings"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0]["bindingType"], "develop");

    // 更新线索状态为 resolved
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "update", &thread_id, "--status", "resolved"]);
    cmd.assert().success();
}

#[test]
fn e2e_emergence_accept_full_lifecycle() {
    // 创建涌现 → 评分 → 决策 accept → 执行到章节 → 归档
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // create
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "意外伏笔",
        "--description", "一个意外的情节机会",
        "--chapter", &ch_id,
        "--json",
    ]);
    let em = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(em["status"], "pending");
    let em_id = em["id"].as_str().unwrap().to_string();

    // rate（pending → rated）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "rate", &em_id, "--rating", "9", "--reason", "高价值", "--json"]);
    let em = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(em["status"], "rated");

    // decide accept（rated → decided）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "decide", &em_id, "--decision", "accept", "--reason", "采纳", "--json"]);
    let em = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(em["status"], "decided");

    // execute（decided → executed）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "已执行", "--json"]);
    let em = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(em["status"], "executed");

    // archive（executed → archived）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "archive", &em_id, "--json"]);
    let em = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(em["status"], "archived");
}

#[test]
fn e2e_emergence_defer_blocks_execution() {
    // 创建涌现 → 评分 → 决策 defer → 验证无法执行
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "暂缓涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    common::novel_at(&dir)
        .args(["emergence", "rate", &em_id, "--rating", "5", "--reason", "理由"])
        .assert().success();
    common::novel_at(&dir)
        .args(["emergence", "decide", &em_id, "--decision", "defer", "--reason", "暂缓"])
        .assert().success();

    // defer 决策无法执行（invalid_state_transition → category 3）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "备注"]);
    cmd.assert().failure().code(3); // invalid_state_transition → category 3
}

#[test]
fn e2e_emergence_reject_blocks_execution() {
    // 创建涌现 → 评分 → 决策 reject → 验证无法执行
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "拒绝涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    common::novel_at(&dir)
        .args(["emergence", "rate", &em_id, "--rating", "2", "--reason", "理由"])
        .assert().success();
    common::novel_at(&dir)
        .args(["emergence", "decide", &em_id, "--decision", "reject", "--reason", "拒绝"])
        .assert().success();

    // reject 决策无法执行（invalid_state_transition → category 3）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "备注"]);
    cmd.assert().failure().code(3); // invalid_state_transition → category 3
}

#[test]
fn e2e_combined_mainplot_thread_emergence() {
    // 综合场景：主线节点 + 线索 + 涌现在一卷内协作
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // 创建主线节点
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "综合主线",
        "--target-chapters", &ch_id,
        "--json",
    ]);
    let node = parse_json(&cmd.assert().success().get_output().stdout);
    let node_id = node["id"].as_str().unwrap().to_string();

    // 创建线索绑定章节
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "综合线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();
    common::novel_at(&dir)
        .args(["thread", "bind", &thread_id, "--chapter", &ch_id, "--type", "develop"])
        .assert().success();

    // 创建涌现并走完整生命周期
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "综合涌现",
        "--description", "描述",
        "--chapter", &ch_id,
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    common::novel_at(&dir).args(["emergence", "rate", &em_id, "--rating", "8", "--reason", "理由"]).assert().success();
    common::novel_at(&dir).args(["emergence", "decide", &em_id, "--decision", "accept", "--reason", "采纳"]).assert().success();
    common::novel_at(&dir).args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "执行"]).assert().success();
    common::novel_at(&dir).args(["emergence", "archive", &em_id]).assert().success();

    // 标记主线节点达成
    common::novel_at(&dir)
        .args(["mainplot", "node", "update", &node_id, "--achieved", "1"])
        .assert().success();

    // 最终验证：卷内有 1 个达成节点、1 条线索、涌现已归档
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "list", &vid.to_string(), "--json"]);
    let nodes = parse_json(&cmd.assert().success().get_output().stdout)["nodes"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["achieved"], 1);

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "list", &vid.to_string(), "--json"]);
    let threads = parse_json(&cmd.assert().success().get_output().stdout)["threads"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(threads, 1);

    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "list", &vid.to_string(), "--status", "archived", "--json"]);
    let archived = parse_json(&cmd.assert().success().get_output().stdout)["emergences"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(archived, 1);
}
