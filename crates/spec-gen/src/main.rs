//! spec-gen：从 openspec spec 的 @codegen 块生成契约测试，构造性消除设计↔实现漂移。
//!
//! 单一事实来源链路：
//!   openspec/specs/**/spec.md（@codegen 块）  ← 唯一人类编写源
//!     → spec_parse::parse_specs()
//!     → ssot::load_ssot() + resolve_expectations()
//!     → render::render_test()
//!     → contracts/cli/generated/<domain>.gen.rs + .gen.golden.json
//!
//! 三道闸门（SSOT 对齐 / fixture 引用 / 生成物漂移）任一失败即非零退出。

mod spec_parse;
mod ssot;
mod render;
mod enum_source;
mod json_schema;
mod md_render;
mod commands_source;
mod registry_source;

use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("generate") => generate(),
        Some("check") => check(),
        Some("enum-source") => enum_source(),
        Some("json-schema") => json_schema_generate(),
        Some("--help") | Some("-h") | None => {
            println!("spec-gen：从 openspec @codegen 块生成契约测试 + 类型化契约源生成");
            println!();
            println!("用法：");
            println!("  spec-gen generate        扫描 openspec/specs/，生成 contracts/cli/generated/");
            println!("                           （SSOT 对齐闸失败或 fixture 引用缺失即非零退出）");
            println!("  spec-gen check           在内存重新生成并与仓库 contracts/cli/generated/ 比对，");
            println!("                           diff 非零即退出 1（drift 检查闸）");
            println!("  spec-gen enum-source     从 novel-cli 的 ExitCode/ErrorCode enum 变体渲染");
            println!("                           exit-codes/error-codes 的机器可导出列对账片段");
            println!("                           （contract-governance-02：enum 类型化契约源，对账手写 md）");
            println!("  spec-gen json-schema     从 novel-cli 的 JsonSchema struct 生成");
            println!("                           contracts/cli/json-schemas/*.json（struct 结构契约源，");
            println!("                           contract-governance-03：draft-07 + additionalProperties:false）");
            Ok(())
        }
        Some(other) => {
            anyhow::bail!("未知子命令 '{other}'，可用：generate / check / enum-source / json-schema（--help 查看帮助）")
        }
    }
}

/// 解析+SSOT+渲染的共享流水线，返回 (repo_root, resolved blocks)。
fn pipeline() -> anyhow::Result<(PathBuf, Vec<render::ResolvedBlock>)> {
    let repo_root = repo_root()?;
    let specs_dir = repo_root.join("openspec/specs");
    let fixtures_dir = repo_root.join("contracts/cli/fixtures");
    let ssot_maps = ssot::load_ssot()?;
    let blocks = spec_parse::parse_specs(&specs_dir, &fixtures_dir)?;
    if blocks.is_empty() {
        anyhow::bail!("未找到任何 @codegen 块——至少一个 spec 需带 @codegen 才能生成");
    }
    let resolved = blocks.iter()
        .map(|b| render::resolve_block(b, &ssot_maps))
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok((repo_root, resolved))
}

/// check：在内存重新生成，与仓库生成物 diff。漂移即 exit 1。
fn check() -> anyhow::Result<()> {
    let (repo_root, resolved) = pipeline()?;
    let generated_dir = repo_root.join("contracts/cli/generated");
    if !generated_dir.is_dir() {
        eprintln!("ℹ️  {} 不存在——无生成物可检查，放行（首次生成前）", generated_dir.display());
        return Ok(());
    }
    let domains = render::group_by_domain(&resolved);
    let mut drift = false;
    for (domain, domain_blocks) in &domains {
        let expected = render::render_test_content(domain, domain_blocks);
        let on_disk_path = generated_dir.join(format!("{domain}.gen.rs"));
        let on_disk = std::fs::read_to_string(&on_disk_path).unwrap_or_default();
        if expected != on_disk {
            drift = true;
            eprintln!("❌ 漂移：{}", on_disk_path.display());
            // 打印前若干差异行（简易 unified diff）。
            print_diff(&on_disk, &expected);
        }
    }
    // 仓库里有但本次未生成的文件（源 spec 已删 @codegen 块）也算漂移。
    let expected_files: std::collections::HashSet<String> =
        domains.iter().map(|(d, _)| format!("{d}.gen.rs")).collect();
    for entry in std::fs::read_dir(&generated_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".gen.rs") && !expected_files.contains(&name) {
            drift = true;
            eprintln!("❌ 漂移：{} 存在于生成目录但源 spec 无对应 @codegen 块", name);
        }
    }
    if drift {
        eprintln!("\n修复：运行 `cargo run -p spec-gen -- generate` 重新生成，然后提交。");
        std::process::exit(1);
    }
    eprintln!("✓ 生成物无漂移（contracts/cli/generated/ 与源 spec @codegen 块一致）");

    // contract-governance-03-struct-schema-gen：json-schemas drift 检查。
    // 内存重新生成 24 份生成化 schema 并与仓库比对，漂移 exit 1（三类手写豁免）。
    let schemas_dir = repo_root.join("contracts/cli/json-schemas");
    let schema_drifts = json_schema::check_drift(&schemas_dir);
    if !schema_drifts.is_empty() {
        eprintln!("❌ json-schema 漂移：{} 份", schema_drifts.len());
        for d in &schema_drifts {
            eprintln!("  - {}", d);
        }
        eprintln!("\n修复：运行 `cargo run -p spec-gen -- json-schema` 重新生成。");
        std::process::exit(1);
    }
    eprintln!("✓ json-schema 生成物无漂移（contracts/cli/json-schemas/ 与 struct 源一致）");

    // contract-governance-04-specgen-rebuild：exit-codes/error-codes 机器列 + commands.md 生成段 drift。
    // md 生成段以 @generated 标记界定，drift 只锁机器列（叙述列手写豁免）。
    let enum_maps = enum_source::load_enum_source();
    let exit_block = md_render::render_exit_codes_block(&enum_maps);
    let error_block = md_render::render_error_codes_block(&enum_maps);
    let mut md_drift = false;
    if md_render::check_a_drift(&repo_root.join("contracts/cli/exit-codes.md"), &exit_block, &[0]) {
        md_drift = true;
        eprintln!("❌ exit-codes.md §A 机器列漂移（退出码数值列）");
    }
    if md_render::check_a_drift(&repo_root.join("contracts/cli/error-codes.md"), &error_block, &[0, 2]) {
        md_drift = true;
        eprintln!("❌ error-codes.md §A 机器列漂移（错误码字符串列 / 对应退出码列）");
    }
    let commands = commands_source::collect_commands();
    let cmd_drifts =
        commands_source::check_commands_drift(&repo_root.join("contracts/cli/commands.md"), &commands);
    if !cmd_drifts.is_empty() {
        md_drift = true;
        eprintln!("❌ commands.md 生成段漂移：{}", cmd_drifts.join("、"));
    }

    // contract-registry-v2-03：命令契约注册表交叉一致性校验（复用 validate_registry 单一核心）。
    // 任一类不一致（未登记命令 / 未知错误 ID / guard 无法解析 / 反向追踪失配）即 drift 红 exit 1。
    let reg_issues = commands_source::check_registry_consistency();
    if !reg_issues.is_empty() {
        md_drift = true;
        eprintln!("❌ 命令契约注册表交叉一致性校验失败：");
        for i in &reg_issues {
            eprintln!("  - {i}");
        }
    }

    // contract-registry-v2-04：commands.md 行为语义/退出码生成段比对 + 三类新生成物整段 diff
    // + md_render 叙述映射键集合校准。
    // 1. commands.md 行为语义/退出码生成段（@generated 标记内）逐字比对。
    let sem_drifts = registry_source::check_semantics_drift(&repo_root.join("contracts/cli/commands.md"));
    if !sem_drifts.is_empty() {
        md_drift = true;
        eprintln!("❌ commands.md 行为语义/退出码生成段漂移：{}", sem_drifts.join("、"));
    }
    // 2. 三类整文件生成物整段 diff（command-error-matrix.md / examples/INDEX.md / contract-tests/INDEX.md）。
    let matrix_path = repo_root.join("contracts/cli/command-error-matrix.md");
    let examples_index_path = repo_root.join("contracts/cli/examples/INDEX.md");
    let test_index_path = repo_root.join("contracts/cli/contract-tests/INDEX.md");
    let full_file_checks = vec![
        (
            matrix_path,
            "command-error-matrix.md",
            registry_source::render_error_matrix(&repo_root),
        ),
        (
            examples_index_path,
            "examples/INDEX.md",
            registry_source::render_examples_index(&repo_root),
        ),
        (
            test_index_path,
            "contract-tests/INDEX.md",
            registry_source::render_test_index(&repo_root),
        ),
    ];
    for (p, label, expected) in full_file_checks {
        if registry_source::check_full_file_drift(&p, &expected) {
            md_drift = true;
            eprintln!("❌ {label} 整段漂移（手改生成物或源未重新生成）");
        }
    }
    // 3. md_render 叙述映射键集合校准（enum 派生集合 ↔ 叙述映射键集合，未知/缺失键即红）。
    let key_issues = registry_source::check_narrative_keys_drift(&enum_maps);
    if !key_issues.is_empty() {
        md_drift = true;
        eprintln!("❌ 叙述映射键集合校准失败：");
        for i in &key_issues {
            eprintln!("  - {i}");
        }
    }

    if md_drift {
        eprintln!("\n修复：运行 `cargo run -p spec-gen -- generate` 重新生成，然后提交。");
        std::process::exit(1);
    }
    eprintln!("✓ md 生成物无漂移（exit-codes/error-codes 机器列 + commands.md 生成段）");
    eprintln!("✓ 注册表驱动生成物无漂移（commands.md 行为语义/退出码段 + command-error-matrix + 双索引整段）");
    eprintln!("✓ 叙述映射键集合校准通过（exit/error 叙述键集合 ⊆ enum 派生集合）");
    eprintln!("✓ 命令契约注册表交叉一致性通过（clap 叶子 ↔ 注册表 ↔ ErrorCatalog）");
    Ok(())
}

/// 简易 diff：逐行输出首个差异起的后若干行。
fn print_diff(a: &str, b: &str) {
    let av: Vec<&str> = a.lines().collect();
    let bv: Vec<&str> = b.lines().collect();
    let mut i = 0;
    while i < av.len().min(bv.len()) && av[i] == bv[i] { i += 1; }
    eprintln!("  首个差异行 {}（共 {} vs {} 行）：", i + 1, av.len(), bv.len());
    for k in i..av.len().min(i + 6) {
        eprintln!("  - {}", av[k]);
    }
    for k in i..bv.len().min(i + 6) {
        eprintln!("  + {}", bv[k]);
    }
}

/// 扫描 spec → 解析 SSOT → 解析 @codegen → 解析期望 → 渲染 → 写生成物 → 注册 [[test]]。
fn generate() -> anyhow::Result<()> {
    let repo_root = repo_root()?;
    let specs_dir = repo_root.join("openspec/specs");
    let generated_dir = repo_root.join("contracts/cli/generated");
    let fixtures_dir = repo_root.join("contracts/cli/fixtures");
    let novel_cli_cargo = repo_root.join("crates/novel-cli/Cargo.toml");

    // 1. 加载 SSOT（exit-codes.md / error-codes.md）。
    let ssot_maps = ssot::load_ssot()?;
    eprintln!("[ssot] 退出码映射 {} 条，错误码映射 {} 条",
        ssot_maps.exit_codes.len(), ssot_maps.error_codes.len());

    // 2. 解析全部 spec 的 @codegen 块（fixture 引用闸在此校验）。
    let blocks = spec_parse::parse_specs(&specs_dir, &fixtures_dir)?;
    eprintln!("[spec] 解析到 {} 个 @codegen 块", blocks.len());
    if blocks.is_empty() {
        anyhow::bail!("未找到任何 @codegen 块——至少一个 spec 需带 @codegen 才能生成");
    }

    // 3. 解析期望值（SSOT 对齐闸在此校验：outcome 查不到即 bail）。
    let resolved = blocks.iter()
        .map(|b| render::resolve_block(b, &ssot_maps))
        .collect::<anyhow::Result<Vec<_>>>()?;
    eprintln!("[resolve] 全部 {} 块的期望值经 SSOT 解析通过", resolved.len());

    // 4. 按 domain 分组渲染生成物。
    std::fs::create_dir_all(&generated_dir)?;
    let domains = render::group_by_domain(&resolved);
    for (domain, domain_blocks) in &domains {
        let test_path = generated_dir.join(format!("{domain}.gen.rs"));
        render::render_test_file(domain, domain_blocks, &test_path)?;
        eprintln!("[render] 写出 {}", test_path.display());
    }

    // 5. 注册 [[test]] 到 novel-cli Cargo.toml。
    let registered = render::register_tests(&novel_cli_cargo, &domains)?;
    if !registered.is_empty() {
        eprintln!("[register] 注册 [[test]]: {}", registered.join(", "));
    }

    // 6. 生成 exit-codes.md / error-codes.md §A 整表（机器列 + 叙述列，contract-governance-04）。
    let enum_maps = enum_source::load_enum_source();
    let exit_block = md_render::render_exit_codes_block(&enum_maps);
    let error_block = md_render::render_error_codes_block(&enum_maps);
    md_render::apply_a_table(&repo_root.join("contracts/cli/exit-codes.md"), exit_block)?;
    eprintln!("[md] 生成 exit-codes.md §A 整表");
    md_render::apply_a_table(&repo_root.join("contracts/cli/error-codes.md"), error_block)?;
    eprintln!("[md] 生成 error-codes.md §A 整表");

    // 7. 生成 commands.md 语法段 + 参数表（clap 命令树，contract-governance-04）。
    let commands = commands_source::collect_commands();
    eprintln!("[md] 遍历到 {} 个叶子命令", commands.len());
    commands_source::apply_commands_md(&repo_root.join("contracts/cli/commands.md"), &commands)?;
    eprintln!("[md] 生成 commands.md 语法段 + 参数表");

    // 8. 命令契约注册表交叉一致性校验（contract-registry-v2-03）：复用 validate_registry
    //    单一核心；generate 路径同样强制——不一致即阻断生成（spec「任一类异常 MUST 阻断」）。
    let reg_issues = commands_source::check_registry_consistency();
    if !reg_issues.is_empty() {
        eprintln!("❌ 命令契约注册表交叉一致性校验失败：");
        for i in &reg_issues {
            eprintln!("  - {i}");
        }
        anyhow::bail!("命令契约注册表与 clap 叶子 / ErrorCatalog 不一致，生成已阻断");
    }
    eprintln!("[registry] 命令契约注册表交叉一致性通过（clap 叶子 ↔ 注册表 ↔ ErrorCatalog）");

    // 9. 注册表驱动完整契约生成（contract-registry-v2-04）：命令错误矩阵、示例/契约测试
    //    索引、commands.md 行为语义 + 退出码段（注册表 + ErrorCatalog 派生）。
    //    反向一致性校验在最后执行——不一致即阻断生成（spec「任一类异常 MUST 阻断」）。
    let matrix = registry_source::render_error_matrix(&repo_root);
    let matrix_path = repo_root.join("contracts/cli/command-error-matrix.md");
    std::fs::write(&matrix_path, &matrix)?;
    eprintln!("[registry-source] 写出 {}", matrix_path.display());

    let examples_index = registry_source::render_examples_index(&repo_root);
    let examples_index_path = repo_root.join("contracts/cli/examples/INDEX.md");
    std::fs::write(&examples_index_path, &examples_index)?;
    eprintln!("[registry-source] 写出 {}", examples_index_path.display());

    let test_index = registry_source::render_test_index(&repo_root);
    let test_index_path = repo_root.join("contracts/cli/contract-tests/INDEX.md");
    std::fs::write(&test_index_path, &test_index)?;
    eprintln!("[registry-source] 写出 {}", test_index_path.display());

    // commands.md 行为语义 + 退出码段替换（读回语法段生成后的文件，套注册表驱动生成段）。
    let commands_md_path = repo_root.join("contracts/cli/commands.md");
    let commands_content = std::fs::read_to_string(&commands_md_path)?;
    let semantics_applied = registry_source::apply_command_semantics_blocks(&commands_content);
    if semantics_applied != commands_content {
        std::fs::write(&commands_md_path, &semantics_applied)?;
        eprintln!("[registry-source] 替换 commands.md 行为语义 + 退出码段（注册表驱动）");
    } else {
        eprintln!("[registry-source] commands.md 行为语义 + 退出码段无需变更");
    }

    // 反向一致性校验（单一函数，design D5）。
    let issues = registry_source::check_index_consistency(&repo_root);
    if !issues.is_empty() {
        eprintln!("❌ 反向一致性校验失败：");
        for i in &issues {
            eprintln!("  - {i}");
        }
        anyhow::bail!("注册表/ErrorCatalog/文件系统资产一致性不一致，生成已阻断");
    }
    eprintln!("[registry-source] 反向一致性校验通过（schema 引用 / 错误矩阵 / 测试注册 / 示例追溯）");
    Ok(())
}

/// 仓库根：从 CARGO_MANIFEST_DIR 向上到含 contracts/ 与 openspec/ 的目录。
fn repo_root() -> anyhow::Result<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while dir.is_dir() {
        if dir.join("contracts/cli").is_dir() && dir.join("openspec").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    anyhow::bail!("无法定位仓库根（需含 contracts/cli/ 与 openspec/）")
}

/// json-schema：从 novel-cli 的 JsonSchema struct 生成 contracts/cli/json-schemas/*.json
/// （contract-governance-03-struct-schema-gen）。
///
/// 生成范围：24 份由 struct 派生的 schema（setting/constraint/thread/mainplot/emergence/
/// volume/export/verify/info 各族）。`chapter`/`verdict`/`chapter-context` 三类保留手写
/// （renderer 用 json! 宏无 struct，design Decision 6）。
///
/// 生成物特性：`$schema=draft-07`（与 schemars 0.8 原生一致）、object 节点
/// `additionalProperties:false`（对齐手写版封闭语义）。字段集以 struct 源为准——
/// 生成物与手写版若有偏差，按 spec「以 struct 源为准修正生成物」处理，不改 struct
/// 字段定义、不改契约测试断言。
fn json_schema_generate() -> anyhow::Result<()> {
    let repo_root = repo_root()?;
    let schemas_dir = repo_root.join("contracts/cli/json-schemas");
    if !schemas_dir.is_dir() {
        anyhow::bail!("{} 不存在——无法生成 json-schemas", schemas_dir.display());
    }

    let mappings = json_schema::mappings();
    eprintln!("[json-schema] 生成 {} 份 schema", mappings.len());

    for mapping in mappings {
        let content = json_schema::render_schema_string(mapping);
        let path = schemas_dir.join(format!("{}.schema.json", mapping.name));
        std::fs::write(&path, &content)?;
        eprintln!("[json-schema] 写出 {}", path.display());
    }

    // 对账报告：stdout 输出映射表，供人比对 struct 源与生成物。
    println!("{}", json_schema::report());
    Ok(())
}

/// enum-source：从 novel-cli 的 ExitCode/ErrorCode enum 变体渲染 exit-codes/error-codes 的
/// 机器可导出列对账片段，写入 contracts/cli/generated/enum-source/（contract-governance-02）。
///
/// 生成范围（design Q1）：只生成机器可导出的数值与字符串列，不生成中文「类别/含义/固化切片」
/// 叙述列（仍手写）。生成物用于与手写 `exit-codes.md` §A / `error-codes.md` §A 对账：
/// 手写 md 的退出码数值 / 错误码字符串集合若与 enum 源不一致，视为 md 漂移，按
/// 「以 errors/mod.rs::exit_code() 为权威改 md」修正。不覆盖手写 md 全表。
///
/// 不动既有 `@codegen`→契约测试生成链路（generate / check 保持不变）。
fn enum_source() -> anyhow::Result<()> {
    let repo_root = repo_root()?;
    let maps = enum_source::load_enum_source();
    eprintln!(
        "[enum-source] ExitCode 变体 {} 条，ErrorCode 变体 {} 条",
        maps.exit_codes.len(),
        maps.error_codes.len()
    );

    let out_dir = repo_root.join("contracts/cli/generated/enum-source");
    std::fs::create_dir_all(&out_dir)?;

    let exit_fragment = maps.render_exit_codes_fragment();
    let exit_path = out_dir.join("exit-codes.fragment.md");
    std::fs::write(&exit_path, &exit_fragment)?;
    eprintln!("[enum-source] 写出 {}", exit_path.display());

    let err_fragment = maps.render_error_codes_fragment();
    let err_path = out_dir.join("error-codes.fragment.md");
    std::fs::write(&err_path, &err_fragment)?;
    eprintln!("[enum-source] 写出 {}", err_path.display());

    // 对账报告：stdout 输出机器可导出集合，供人比对手写 md。
    println!("# enum-source 对账报告\n");
    println!("## exit-codes（ExitCode 判别式，{} 条）\n", maps.exit_codes.len());
    print!("{}", exit_fragment);
    println!("\n## error-codes（ErrorCode serialize，{} 条）\n", maps.error_codes.len());
    print!("{}", err_fragment);
    Ok(())
}
