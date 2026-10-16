#!/usr/bin/env bash
# novel-cli 端到端演示脚本
#
# 用真实 `novel` CLI 子命令完整跑通一个小说项目的生命周期，验证硬层命令链可跑通。
# 不直接读写 SQLite / `.novel/content`，只通过 `novel` 子命令访问世界状态
# （CLAUDE.md §3、§8）。
#
# 覆盖的命令链：
#   init → setting set/get/list → constraint define/render/list/check
#   → chapter create/update/get/list/status(→approved)
#   → mainplot node create/get/list/update → verdict set-mainplot-landing
#   → thread create/bind/query/update/list
#   → emergence create/rate/decide/execute/archive/list（完整生命周期 + defer/reject 阻断）
#   → volume seal/archive → export full/volume → verify integrity
#
# 用法：
#   ./scripts/run-novel-cli-e2e.sh            # 用 cargo build 出的二进制
#   ./scripts/run-novel-cli-e2e.sh --bin /path/to/novel   # 用指定二进制
#   ./scripts/run-novel-cli-e2e.sh --keep     # 跑完保留临时世界目录
#
# 退出码：0 表示全流程跑通；非 0 表示某步失败（脚本会在失败处停止）。

set -euo pipefail

# ---------- 初始化与参数解析 ----------

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY=""
KEEP=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin)
      BINARY="$2"; shift 2 ;;
    --keep)
      KEEP=true; shift ;;
    -h|--help)
      sed -n '2,30p' "$0"
      exit 0 ;;
    *)
      echo "未知参数: $1" >&2; exit 2 ;;
  esac
done

# 定位 novel 二进制：优先 --bin，其次 cargo build 产物，最后尝试 cargo run。
if [[ -n "$BINARY" ]]; then
  NOVEL="$BINARY"
elif [[ -x "$REPO_ROOT/target/debug/novel" ]]; then
  NOVEL="$REPO_ROOT/target/debug/novel"
else
  echo "未找到已编译的 novel 二进制，执行 cargo build -p novel-cli ..." >&2
  (cd "$REPO_ROOT" && cargo build -p novel-cli)
  NOVEL="$REPO_ROOT/target/debug/novel"
fi

command -v jq >/dev/null || { echo "需要 jq 来解析 JSON 输出，请先安装 jq" >&2; exit 2; }

# 颜色输出（仅终端场景；不污染 novel 的 --json 输出，因为颜色只用在 echo）。
if [[ -t 1 ]]; then
  C_GREEN=$'\033[32m'; C_RED=$'\033[31m'; C_CYAN=$'\033[36m'; C_DIM=$'\033[2m'; C_RESET=$'\033[0m'
else
  C_GREEN=''; C_RED=''; C_CYAN=''; C_DIM=''; C_RESET=''
fi

PASS=0
FAIL=0

# step <描述> <命令...>：执行一条 novel 命令，检查退出码；失败则打印输出并停止。
# 通过环境变量 STEP_STDOUT / STEP_STDERR 拿到输出（供调用方解析）。
STEP_STDOUT=""
STEP_STDERR=""
step() {
  local desc="$1"; shift
  echo "${C_CYAN}▶ ${desc}${C_RESET}"
  echo "${C_DIM}  $ $*${C_RESET}"
  # 临时文件捕获输出，避免 set -e 在管道里吞掉退出码。
  local out_err
  out_err="$(mktemp)"
  local rc=0
  "$@" >"$out_err" 2>"$out_err" || rc=$?
  if [[ "$rc" -eq 0 ]]; then
    STEP_STDOUT="$(cat "$out_err")"
    STEP_STDERR=""
    rm -f "$out_err"
    PASS=$((PASS + 1))
    echo "${C_GREEN}  ✓ 成功${C_RESET}"
  else
    STEP_STDOUT="$(cat "$out_err")"
    STEP_STDERR="$STEP_STDOUT"
    rm -f "$out_err"
    FAIL=$((FAIL + 1))
    echo "${C_RED}  ✗ 失败（退出码 ${rc}）${C_RESET}" >&2
    echo "${C_RED}---- 输出 ----${C_RESET}" >&2
    cat >&2 <<<"$STEP_STDOUT"
    echo "${C_RED}-------------${C_RESET}" >&2
    echo "${C_RED}端到端流程在第 $((PASS + FAIL)) 步失败，停止。${C_RESET}" >&2
    if [[ "$KEEP" == "false" ]]; then
      echo "${C_DIM}（保留临时目录以排查：如需保留请加 --keep）${C_RESET}" >&2
    fi
    exit 1
  fi
}

# 期望失败：用于验证硬阻断（如 defer/reject 涌现无法 execute）。
# expect_fail <描述> <期望退出码> <命令...>
# 严格校验：命令必须失败，且退出码必须等于期望值，否则视为契约不符。
expect_fail() {
  local desc="$1"; local expect_rc="$2"; shift 2
  echo "${C_CYAN}▶ ${desc}（期望失败，退出码 ${expect_rc}）${C_RESET}"
  echo "${C_DIM}  $ $*${C_RESET}"
  local out_err
  out_err="$(mktemp)"
  local rc=0
  set +e
  "$@" >"$out_err" 2>"$out_err" || rc=$?
  set -e
  local captured; captured="$(cat "$out_err")"; rm -f "$out_err"
  if [[ "$rc" -eq 0 ]]; then
    FAIL=$((FAIL + 1))
    echo "${C_RED}  ✗ 本应失败但成功了${C_RESET}" >&2
    cat >&2 <<<"$captured"
    exit 1
  elif [[ "$rc" -ne "$expect_rc" ]]; then
    FAIL=$((FAIL + 1))
    echo "${C_RED}  ✗ 失败但退出码不符契约：期望 ${expect_rc}，实际 ${rc}${C_RESET}" >&2
    cat >&2 <<<"$captured"
    exit 1
  else
    PASS=$((PASS + 1))
    echo "${C_GREEN}  ✓ 如预期失败（退出码 ${rc}）${C_RESET}"
  fi
}

# ---------- 准备临时世界目录 ----------

WORK_DIR="$(mktemp -d -t novel-e2e-XXXXXX)"
echo "${C_CYAN}=== novel-cli 端到端演示 ===${C_RESET}"
echo "${C_DIM}二进制: $NOVEL${C_RESET}"
echo "${C_DIM}世界目录: $WORK_DIR${C_RESET}"
if [[ "$KEEP" == "true" ]]; then
  echo "${C_DIM}（--keep：跑完保留目录）${C_RESET}"
fi
echo

# 清理函数：跑完或中断时清理临时目录（--keep 除外）。
cleanup() {
  local rc=$?
  echo
  echo "${C_CYAN}=== 汇总 ===${C_RESET}"
  echo "${C_GREEN}成功步骤: $PASS${C_RESET}"
  if [[ "$FAIL" -gt 0 ]]; then
    echo "${C_RED}失败步骤: $FAIL${C_RESET}"
  fi
  if [[ "$KEEP" == "false" ]]; then
    rm -rf "$WORK_DIR"
    echo "${C_DIM}已清理临时目录: $WORK_DIR${C_RESET}"
  else
    echo "${C_DIM}保留临时目录: $WORK_DIR${C_RESET}"
  fi
  exit $rc
}
trap cleanup EXIT INT TERM

# 所有命令统一通过 --path 指向世界目录（chapter 组子命令已补齐 --path，无需 cd 子壳变通）。
novel() { "$NOVEL" "$@"; }

# ====================================================================
# 1. 项目初始化
# ====================================================================
echo "${C_CYAN}── 1. 项目初始化 ─${C_RESET}"
step "初始化世界 demo-world" \
  novel init demo-world --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | head -5 | sed 's/^/  /'

step "查看项目信息" \
  novel info --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  项目: \(.name) | schemaVersion: \(.schemaVersion)"' 2>/dev/null || head -5 <<<"$STEP_STDOUT" | sed 's/^/  /'

# ====================================================================
# 2. 配置项 setting
# ====================================================================
echo
echo "${C_CYAN}── 2. 配置项 setting ─${C_RESET}"
step "设置 world-setting 配置项" \
  novel setting set world-setting "东方奇幻世界，灵气复苏后的第三纪元" --path "$WORK_DIR"

step "查询 world-setting 配置项（JSON）" \
  novel setting get world-setting --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  \(.name) = \(.value)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "列出所有配置项" \
  novel setting list --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.settings | length) 个配置项"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 3. 约束规则 constraint
# ====================================================================
echo
echo "${C_CYAN}── 3. 约束规则 constraint ─${C_RESET}"
step "定义 plot 类型的 hard 约束 时间线一致性" \
  novel constraint define \
    --type plot --scope world --name 时间线一致性 \
    --content "所有章节的时间线必须前后一致，不得出现倒错" \
    --severity hard --path "$WORK_DIR"

step "定义 style 类型的 info 约束 视角限制" \
  novel constraint define \
    --type style --scope world --name 视角限制 \
    --content "默认采用第三人称限制视角" \
    --severity info --path "$WORK_DIR"

step "查询单个约束（render）" \
  novel constraint render --type plot --scope world --name 时间线一致性 --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  \(.constraintType)/\(.severity): \(.content)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "列出所有约束" \
  novel constraint list --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.constraints | length) 条约束"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 4. 卷与章节
# ====================================================================
echo
echo "${C_CYAN}── 4. 卷与章节 ─${C_RESET}"
step "创建第一卷" \
  novel volume create 卷一 --description "开篇卷" --order 1 --path "$WORK_DIR"
# volume create 不支持 --json，从人读输出解析 "ID = N"。
VOLUME_ID=$(grep -oE 'ID = [0-9]+' <<<"$STEP_STDOUT" | grep -oE '[0-9]+' | head -1)
echo "  卷 ID = $VOLUME_ID"

step "查询卷信息" \
  novel volume info "$VOLUME_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  \(.name) | status=\(.status) | order=\(.order)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "列出所有卷" \
  novel volume list --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.volumes | length) 个卷"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "创建第一章" \
  novel chapter create 第一章 --json --path "$WORK_DIR"
CHAPTER_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  章节 ID = $CHAPTER_ID"

step "创建第二章" \
  novel chapter create 第二章 --json --path "$WORK_DIR"
CHAPTER2_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  章节 ID = $CHAPTER2_ID"

step "列出所有章节" \
  novel chapter list --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.chapters | length) 个章节"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# 写入第一章正文（正文文件在临时工作区，路径相对当前目录）。
CHAPTER1_CONTENT="$WORK_DIR/ch1-draft.md"
cat >"$CHAPTER1_CONTENT" <<'EOF'
第一章 启程

清晨的雾气还未散尽，少年推开木门，望向远处连绵的山脊。
灵气复苏三年，世界早已不是旧日模样。他紧了紧背上的行囊，
迈出第一步——这条通往北境的路，他已等了太久。
EOF

step "更新第一章正文" \
  novel chapter update "$CHAPTER_ID" --content "$CHAPTER1_CONTENT" --path "$WORK_DIR"

step "读取第一章元数据与正文（拿 contentHash）" \
  novel chapter get "$CHAPTER_ID" --json --path "$WORK_DIR"
CONTENT_HASH=$(jq -r '.contentHash' <<<"$STEP_STDOUT")
WORD_COUNT=$(jq -r '.wordCount' <<<"$STEP_STDOUT")
echo "  contentHash = $CONTENT_HASH"
echo "  wordCount = $WORD_COUNT"

step "验证章节正文 hash 一致性（list --verify-hash）" \
  novel chapter list --json --verify-hash --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  hashVerified: \(.chapters[0].hashVerified) | hashMismatch: \(.chapters[0].hashMismatch)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 5. 主线节点 mainplot + 主线落地裁决 verdict
# ====================================================================
echo
echo "${C_CYAN}── 5. 主线节点 mainplot + 裁决 verdict ─${C_RESET}"
step "创建主线节点（关联第一章）" \
  novel mainplot node create "$VOLUME_ID" \
    --title "第一章主角启程" \
    --description "主角离开故乡踏上旅途" \
    --target-chapters "$CHAPTER_ID" \
    --json --path "$WORK_DIR"
NODE_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  节点 ID = $NODE_ID"

step "查询主线节点" \
  novel mainplot node get "$NODE_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  \(.title) | achieved=\(.achieved)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "列举卷内主线节点" \
  novel mainplot node list "$VOLUME_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.nodes | length) 个节点"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# 写入主线落地裁决（achieved=1），绑定当前正文 hash。verdict 的 node_id 为字符串。
step "记录主线落地裁决（achieved=1）" \
  novel verdict set-mainplot-landing "$CHAPTER_ID" \
    --node-id "$NODE_ID" --achieved 1 --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  chapter=\(.chapterId) | node=\(.nodeId) | achieved=\(.achieved)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "标记主线节点达成（mainplot node update --achieved 1）" \
  novel mainplot node update "$NODE_ID" --achieved 1 --path "$WORK_DIR"

# ====================================================================
# 6. 规则检查 constraint check（写入通过的 hard 检查，绑定当前 hash）
# ====================================================================
echo
echo "${C_CYAN}── 6. 规则检查 constraint check ─${C_RESET}"
# 构造检查结果输入 JSON（camelCase 字段，contentHash 必须与当前章节一致）。
CHECK_INPUT="$WORK_DIR/check-input.json"
cat >"$CHECK_INPUT" <<EOF
{
  "contentHash": "$CONTENT_HASH",
  "checks": [
    {
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "时间线一致性",
      "severity": "hard",
      "passed": true,
      "description": "第一章时间线与前序一致",
      "location": null
    },
    {
      "constraintType": "style",
      "constraintScope": "world",
      "constraintName": "视角限制",
      "severity": "info",
      "passed": true,
      "description": "采用第三人称限制视角",
      "location": null
    }
  ]
}
EOF

step "批量写入规则检查结果（constraint check --input）" \
  novel constraint check --chapter "$CHAPTER_ID" --input "$CHECK_INPUT" --json --path "$WORK_DIR"

step "查询章节规则检查结果（constraint check --list）" \
  novel constraint check --chapter "$CHAPTER_ID" --list --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.checks | length) 条检查结果"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "验证检查结果 hash 一致性（--list --verify-hash）" \
  novel constraint check --chapter "$CHAPTER_ID" --list --verify-hash --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  检查结果数: \(.checks | length) | stale 数: \(.checks | map(select(.stale)) | length)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 7. 章节状态机推进 outline → drafting → review → approved
# ====================================================================
echo
echo "${C_CYAN}── 7. 章节状态机推进 ─${C_RESET}"
step "推进第一章状态 outline → drafting" \
  novel chapter status "$CHAPTER_ID" drafting --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  → \(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "推进第一章状态 drafting → review（门禁：正文+裁决+hard 检查全过）" \
  novel chapter status "$CHAPTER_ID" review --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  → \(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "推进第一章状态 review → approved（终态）" \
  novel chapter status "$CHAPTER_ID" approved --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  → \(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# 验证终态冻结：approved 后不得再推进状态（退出码 3 = precondition_blocked）。
expect_fail "验证 approved 终态冻结（approved → drafting 应被阻断）" 3 \
  novel chapter status "$CHAPTER_ID" drafting --path "$WORK_DIR"

# ====================================================================
# 8. 线索 thread（多章节绑定）
# ====================================================================
echo
echo "${C_CYAN}── 8. 线索 thread ─${C_RESET}"
step "创建线索 复仇线" \
  novel thread create "$VOLUME_ID" --name 复仇线 \
    --description "主角为师门复仇的暗线" --json --path "$WORK_DIR"
THREAD_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  线索 ID = $THREAD_ID"

step "绑定线索到第一章（mention）" \
  novel thread bind "$THREAD_ID" --chapter "$CHAPTER_ID" --type mention \
    --note "首次提及师门旧事" --path "$WORK_DIR"

step "绑定线索到第二章（develop）" \
  novel thread bind "$THREAD_ID" --chapter "$CHAPTER2_ID" --type develop \
    --note "线索进一步推进" --path "$WORK_DIR"

step "查询第一章的线索绑定" \
  novel thread query --chapter "$CHAPTER_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  \(.bindings | length) 条绑定；类型=\(.bindings[0].bindingType)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "更新线索状态为 resolved" \
  novel thread update "$THREAD_ID" --status resolved --path "$WORK_DIR"

step "列举卷内线索" \
  novel thread list "$VOLUME_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  共 \(.threads | length) 条线索"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 9. 涌现 emergence 完整生命周期（accept 路径）
# ====================================================================
echo
echo "${C_CYAN}── 9. 涌现 emergence（accept 全生命周期） ─${C_RESET}"
# 主卷里只走 accept 全生命周期并归档，使涌现进入终态 archived，满足后续封卷前置。
# defer / reject 的阻断演示放在 §12 用独立小世界进行，避免污染主卷封卷。
step "创建涌现 意外伏笔（关联第一章）" \
  novel emergence create "$VOLUME_ID" \
    --title 意外伏笔 --description "一个意外的情节机会" \
    --chapter "$CHAPTER_ID" --json --path "$WORK_DIR"
EM_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  涌现 ID = $EM_ID"

step "涌现评分（pending → rated）" \
  novel emergence rate "$EM_ID" --rating 9 --reason "高价值伏笔" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  status=\(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "涌现决策 accept（rated → decided）" \
  novel emergence decide "$EM_ID" --decision accept --reason "采纳并执行" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  status=\(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "涌现执行（decided → executed）" \
  novel emergence execute "$EM_ID" --target "$CHAPTER_ID" --note "已并入第一章伏笔" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  status=\(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "涌现归档（executed → archived）" \
  novel emergence archive "$EM_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  status=\(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "列举卷内涌现（按状态过滤 archived）" \
  novel emergence list "$VOLUME_ID" --status archived --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  archived 涌现: \(.emergences | length) 个"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 10. 封卷 volume seal + 归档 archive
# ====================================================================
echo
echo "${C_CYAN}── 10. 封卷与归档 ─${C_RESET}"
# 封卷前置：卷内所有章节 approved + 所有涌现已处置 + 必达目标达成。
# 第一章已 approved，第二章仍 outline → 需先推进到 approved。
# 主卷涌现已全部 archived（终态），不会阻断封卷。
step "推进第二章状态 outline → drafting" \
  novel chapter status "$CHAPTER2_ID" drafting --path "$WORK_DIR"

# 第二章需要正文、裁决、检查才能进 review/approved。先补正文。
CHAPTER2_CONTENT="$WORK_DIR/ch2-draft.md"
echo "第二章 伏笔初现" >"$CHAPTER2_CONTENT"
step "更新第二章正文" \
  novel chapter update "$CHAPTER2_ID" --content "$CHAPTER2_CONTENT" --path "$WORK_DIR"

step "读取第二章 contentHash" \
  novel chapter get "$CHAPTER2_ID" --json --path "$WORK_DIR"
CONTENT2_HASH=$(jq -r '.contentHash' <<<"$STEP_STDOUT")
echo "  contentHash = $CONTENT2_HASH"

step "记录第二章主线落地裁决" \
  novel verdict set-mainplot-landing "$CHAPTER2_ID" \
    --node-id "$NODE_ID" --achieved 1 --path "$WORK_DIR"

# 第二章检查结果输入（contentHash 必须与当前正文一致）。
CHECK2_INPUT="$WORK_DIR/check2-input.json"
cat >"$CHECK2_INPUT" <<EOF
{
  "contentHash": "$CONTENT2_HASH",
  "checks": [
    {
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "时间线一致性",
      "severity": "hard",
      "passed": true,
      "description": "第二章时间线一致",
      "location": null
    }
  ]
}
EOF
step "写入第二章规则检查结果" \
  novel constraint check --chapter "$CHAPTER2_ID" --input "$CHECK2_INPUT" --path "$WORK_DIR"

step "推进第二章状态 drafting → review" \
  novel chapter status "$CHAPTER2_ID" review --path "$WORK_DIR"

step "推进第二章状态 review → approved" \
  novel chapter status "$CHAPTER2_ID" approved --path "$WORK_DIR"

step "封卷 dry-run（前置条件检查，不修改状态）" \
  novel volume seal "$VOLUME_ID" --dry-run --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '.[] | "  【\(.checkType)】\(.passed) — \(.message)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "封卷（推进到 sealing）" \
  novel volume seal "$VOLUME_ID" --json --path "$WORK_DIR"

step "归档卷（推进到 archived，生成卷末监测报告）" \
  novel volume archive "$VOLUME_ID" --path "$WORK_DIR"

step "查询卷信息（确认 archived）" \
  novel volume info "$VOLUME_ID" --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  status=\(.status)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 11. 导出 export + 完整性验证 verify
# ====================================================================
echo
echo "${C_CYAN}── 11. 导出与完整性验证 ─${C_RESET}"
step "导出整卷快照（export volume）" \
  novel export volume "$VOLUME_ID" --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  scope=\(.scope) | warnings=\(.warnings | length) | chapters=\(.chapters | length)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "导出全量快照（export full）" \
  novel export full --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  scope=\(.scope) | volumes=\(.volumes | length) | warnings=\(.warnings | length)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "导出单章节快照（export chapter）" \
  novel export chapter "$CHAPTER_ID" --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  scope=\(.scope) | chapter=\(.chapter.id)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

step "完整性验证 verify integrity" \
  novel verify integrity --json --path "$WORK_DIR"
echo "$STEP_STDOUT" | jq -r '"  hashIntegrity: \(.hashIntegrity.passed) | referenceIntegrity: \(.referenceIntegrity.passed) | stateConsistency: \(.stateConsistency.passed)"' 2>/dev/null || sed 's/^/  /' <<<"$STEP_STDOUT"

# ====================================================================
# 12. 涌现阻断演示（defer / reject 决策无法执行，独立小世界）
# ====================================================================
# defer/reject 涌现无法 execute → 无法 archive，会阻断所在卷的封卷。
# 因此放在主卷封卷之后，用独立小世界演示，不污染主卷。
echo
echo "${C_CYAN}── 12. 涌现阻断演示（defer / reject，独立世界） ─${C_RESET}"
DEMO_DIR="$(mktemp -d -t novel-emg-demo-XXXXXX)"
step "初始化阻断演示小世界" \
  novel init emergence-demo --json --path "$DEMO_DIR"
step "创建演示卷" \
  novel volume create 演示卷 --path "$DEMO_DIR"
DEMO_VID=$(grep -oE 'ID = [0-9]+' <<<"$STEP_STDOUT" | grep -oE '[0-9]+' | head -1)
step "创建演示章节（供 execute --target 使用）" \
  novel chapter create 演示章 --json --path "$DEMO_DIR"
DEMO_CH=$(jq -r '.id' <<<"$STEP_STDOUT")
echo "  演示卷 ID = $DEMO_VID | 演示章节 ID = $DEMO_CH"

# defer 阻断
step "创建 defer 涌现" \
  novel emergence create "$DEMO_VID" --title 暂缓涌现 --description 暂缓 --json --path "$DEMO_DIR"
EM_DEFER_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
step "defer 涌现评分" \
  novel emergence rate "$EM_DEFER_ID" --rating 5 --reason 中等 --path "$DEMO_DIR"
step "defer 涌现决策 defer" \
  novel emergence decide "$EM_DEFER_ID" --decision defer --reason 留待后续 --path "$DEMO_DIR"
expect_fail "验证 defer 决策无法执行（退出码 41）" 41 \
  novel emergence execute "$EM_DEFER_ID" --target "$DEMO_CH" --note 不应执行 --path "$DEMO_DIR"

# reject 阻断
step "创建 reject 涌现" \
  novel emergence create "$DEMO_VID" --title 拒绝涌现 --description 拒绝 --json --path "$DEMO_DIR"
EM_REJECT_ID=$(jq -r '.id' <<<"$STEP_STDOUT")
step "reject 涌现评分" \
  novel emergence rate "$EM_REJECT_ID" --rating 2 --reason 价值低 --path "$DEMO_DIR"
step "reject 涌现决策 reject" \
  novel emergence decide "$EM_REJECT_ID" --decision reject --reason 不采纳 --path "$DEMO_DIR"
expect_fail "验证 reject 决策无法执行（退出码 41）" 41 \
  novel emergence execute "$EM_REJECT_ID" --target "$DEMO_CH" --note 不应执行 --path "$DEMO_DIR"

# 清理阻断演示小世界（独立于主世界 WORK_DIR）。
[[ "$KEEP" == "false" ]] && rm -rf "$DEMO_DIR"

echo
echo "${C_GREEN}=== novel-cli 端到端流程全部跑通 ===${C_RESET}"
