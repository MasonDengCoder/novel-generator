#!/usr/bin/env python3
"""
Progress Leak Guard - 防止进度内容泄漏到设计文件

拦截目标：
进度内容（切片归档时间线、命令实现矩阵、未实现实体清单、schemaVersion 当前值、
状态快照句"已实现 X 条"/"规划态"/"尚未"）只能落 repository/dev-progress.md，
不得出现在 docs/、reference/、README.md、CLAUDE.md。

来源：CLAUDE.md §15 文档变更检查、§13 Do
设计语义（状态机定义、实体职责、§A→§B 升格机制）不属于进度内容，放行。
"""
import sys
import json
import re
import os

# 进度敏感文件：命中这些路径才扫描内容
PROGRESS_SENSITIVE_DIRS = ("docs", "reference")
PROGRESS_SENSITIVE_FILES = ("README.md", "CLAUDE.md")

# 唯一允许承载进度的文件
DEV_PROGRESS = "repository/dev-progress.md"

# 进度泄漏关键词/模式
# 1. 命令实现计数（"已实现 N 条命令"/"46 条已实现"）
COUNT_PATTERN = re.compile(r"已实现\s*\d+\s*(条|命令)|\d+\s*条已实现")

# 2. 实体枚举段（≥3 个规划态实体名连续出现 = 整段列举）
ENTITY_NAMES = ["event", "section", "residue", "phase-log", "character",
                "subplot", "foreshadowing", "payoff", "promise",
                "creation-cycle", "great-cycle"]

# 3. 状态声明快照句
STATUS_PATTERN = re.compile(r"当前为\*\*规划态|当前为规划态门禁|当前为规划阶段|纯文档/规划阶段|0 行实现|实现推进阶段")

# 4. 逐码状态表（✅ 和 ⏳ 同时出现 = 逐码列举已实现/预留状态）
def has_per_code_table(content: str) -> bool:
    return "✅" in content and "⏳" in content


def is_progress_sensitive(path: str) -> bool:
    """判断文件路径是否属进度敏感区（需扫描内容）"""
    if not path:
        return False
    # 归一化
    norm = path.replace("\\", "/").lstrip("./")

    # dev-progress.md 本身放行
    if norm == DEV_PROGRESS or norm.endswith("dev-progress.md"):
        return False

    # contracts/cli/ 与 openspec/specs/ 豁免（§A/§B 是契约分层非进度）
    if "contracts/cli/" in norm or "openspec/specs/" in norm or "openspec/changes/" in norm:
        return False

    # docs/ 或 reference/ 下
    for d in PROGRESS_SENSITIVE_DIRS:
        if norm.startswith(d + "/") or ("/" + d + "/") in norm:
            return True

    # 根级 README.md / CLAUDE.md
    base = norm.split("/")[-1]
    if base in PROGRESS_SENSITIVE_FILES and "/" not in norm.rstrip("/" + base):
        return True
    # 也处理 CLAUDE.md 被全路径引用的情况
    if norm == "CLAUDE.md" or norm == "README.md":
        return True

    return False


def extract_content(event: dict) -> str:
    """从 PreToolUse 事件提取被写入的文本内容"""
    tool_input = event.get("tool_input", {})
    # Edit/MultiEdit 的 new_string / replace；Write 的 content
    parts = []
    if "content" in tool_input:
        parts.append(str(tool_input["content"]))
    if "new_string" in tool_input:
        parts.append(str(tool_input["new_string"]))
    # MultiEdit 的 edits 数组
    edits = tool_input.get("edits", [])
    if isinstance(edits, list):
        for e in edits:
            if isinstance(e, dict) and "new_string" in e:
                parts.append(str(e["new_string"]))
    return "\n".join(parts)


def is_already_pointered(content: str) -> bool:
    """白名单：内容已含 dev-progress 指针视为已指针化，放行（如 Hardness 声明块）"""
    return "repository/dev-progress.md" in content or "dev-progress.md" in content


def detect_leak(content: str) -> str:
    """检测进度泄漏，返回原因描述（无泄漏返回空串）"""
    if COUNT_PATTERN.search(content):
        return "命令实现计数（'已实现 N 条命令'）"

    # 实体枚举段：统计连续出现的规划态实体名数
    hits = sum(1 for name in ENTITY_NAMES if name in content)
    if hits >= 3:
        return f"规划态实体整段枚举（命中 {hits} 个实体名：event/section/residue/phase-log/character/...）"

    if STATUS_PATTERN.search(content):
        return "状态声明快照句（'当前为规划态'/'0 行实现'/'实现推进阶段'）"

    if has_per_code_table(content):
        return "逐码状态表（✅/⏳ 并列的退出码/错误码枚举）"

    return ""


def main():
    try:
        event = json.loads(sys.stdin.read())
    except Exception:
        # 解析失败不阻断（fail-open，避免误伤）
        return

    tool_name = event.get("tool_name", "")
    # 仅 Edit/Write/MultiEdit 触发
    if tool_name not in ("Edit", "Write", "MultiEdit"):
        return

    file_path = event.get("tool_input", {}).get("file_path", "")
    if not file_path:
        return

    if not is_progress_sensitive(file_path):
        return

    content = extract_content(event)
    if not content:
        return

    # 白名单：已指针化的内容放行（分析性参考的诚实标注等）
    if is_already_pointered(content):
        return

    reason = detect_leak(content)
    if reason:
        print(
            f"❌ 进度内容泄漏被拦截：{file_path} 检测到进度快照内容（{reason}）。\n\n"
            "原因：CLAUDE.md §15 要求进度内容只落 repository/dev-progress.md，"
            "设计文件与 CLAUDE.md 不承载状态快照。\n\n"
            "如何继续：\n"
            "1. 若确属进度内容（切片/命令矩阵/未实现实体/schemaVersion 值）"
            "→ 写入 repository/dev-progress.md 对应章节（§3 时间线/§4 命令矩阵/§7.2 未实现实体清单）\n"
            "2. 若属设计语义（状态机定义/实体职责/§A→§B 升格机制）"
            "→ 删状态标记词（已实现/规划态/尚未），保留设计描述\n"
            "3. 指针表述：见 repository/dev-progress.md §N",
            file=sys.stderr
        )
        sys.exit(2)


if __name__ == "__main__":
    main()
