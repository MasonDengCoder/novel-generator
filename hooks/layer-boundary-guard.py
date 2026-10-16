#!/usr/bin/env python3
"""
Layer Boundary Guard - 防止违反软硬层边界

拦截目标：
1. Spec/Skill/Subagent 直接访问 SQLite 或 .novel/content
2. 提供绕过 CLI 的 SDK/API
3. CLI 调度 LLM 或读取 Spec 运行时缓存

来源：CLAUDE.md 第 3、5、13 节
"""
import sys
import json
import re
import os

def main():
    event = json.loads(sys.stdin.read())

    # ✅ 第一步：检查是否在 novel-generator 仓库
    # 本 Hook 仅保护 novel-generator 仓库的边界，不拦截小说项目目录中的合法操作
    cwd = os.getcwd()
    is_novel_generator_root = os.path.basename(cwd) == "novel-generator" or \
                               "novel-generator" in cwd

    if not is_novel_generator_root:
        # 不在 novel-generator 仓库内，放行所有操作
        return

    tool_name = event.get("tool_name")
    if tool_name != "Bash":
        return

    command = event.get("tool_input", {}).get("command", "")

    def is_novel_cli(cmd: str) -> bool:
        # 仅当 `novel` 作为命令词出现（行首或 shell 操作符之后）才视为 CLI 调用，
        # 避免把路径中的 ".novel/" 误判为 CLI 调用。
        return re.search(r"(^|[\n;&|]|&&|\|\|)\s*novel\s+\w", cmd) is not None

    # 拦截直接访问 SQLite（非通过 novel CLI）
    if re.search(r"sqlite3?\s+.*\.novel/novel\.db", command, re.IGNORECASE):
        # 检查是否通过 novel CLI（允许）
        if not is_novel_cli(command):
            print(
                "❌ 危险操作被拦截：直接访问 SQLite 数据库\n\n"
                "原因：CLAUDE.md 规定外部系统只能通过 `novel` 命令访问世界状态。\n\n"
                "如何继续：\n"
                "1. 使用 `novel ... --json` 命令查询数据\n"
                "2. 不要直接运行 `sqlite3 .novel/novel.db`\n"
                "3. CLI Contract 是软硬层唯一稳定边界",
                file=sys.stderr
            )
            sys.exit(2)

    # 拦截直接读取/修改 .novel/content（非通过 novel CLI）
    if re.search(r"(cat|echo|sed|awk|>|>>)\s+.*\.novel/content", command):
        if not is_novel_cli(command):
            print(
                "❌ 危险操作被拦截：直接访问 .novel/content 文件\n\n"
                "原因：CLAUDE.md 规定 Spec 不得直接读取或修改 .novel/content 文件作为状态来源。\n\n"
                "如何继续：\n"
                "1. 使用 `novel chapter get <id> --json` 读取章节内容\n"
                "2. 使用 `novel chapter update <id> --content <file>` 写入章节内容\n"
                "3. 不要绕过 CLI 直接操作正文文件",
                file=sys.stderr
            )
            sys.exit(2)

    # 拦截在 Rust CLI 代码中调用 LLM
    if re.search(r"(anthropic|claude|llm|openai)", command, re.IGNORECASE) and \
       re.search(r"(cargo|rustc|crates/novel-cli)", command):
        print(
            "⚠️  检测到 CLI 代码可能调用 LLM\n\n"
            "CLAUDE.md 规定：\n"
            "「CLI 不调度 LLM、不决定 prompt 形态、不保存 Spec 运行时上下文」\n\n"
            "如果这是：\n"
            "- CLI 内部调用 LLM → 违反架构边界，应移到 Spec 运行层\n"
            "- 安装 anthropic crate 作为依赖 → 确认是否符合分层约束\n"
            "- 注释或文档提及 → 可以继续\n\n"
            "如确认合法，请在 .claude/hooks/layer-boundary-guard.py 中添加例外。",
            file=sys.stderr
        )
        # 警告但不阻断（可能是误报）
        return

if __name__ == "__main__":
    main()
