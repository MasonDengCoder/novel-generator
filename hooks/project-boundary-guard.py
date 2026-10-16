#!/usr/bin/env python3
"""
Project Boundary Guard - 防止在 novel-generator 仓库保存具体小说项目数据

拦截目标：
1. 在本仓库创建 .novel/ 目录
2. 在本仓库写入章节正文、世界数据库
3. 在本仓库写入具体小说项目的运行产物

来源：CLAUDE.md 第 13 节 Don't 第一条
"""
import sys
import json
import re
import os

def main():
    event = json.loads(sys.stdin.read())

    tool_name = event.get("tool_name")
    if tool_name not in ["Write", "Edit", "Bash"]:
        return

    # 获取当前工作目录（应该是 novel-generator 根目录）
    cwd = os.getcwd()
    is_novel_generator_root = os.path.basename(cwd) == "novel-generator" or \
                               "novel-generator" in cwd

    if not is_novel_generator_root:
        # 不在 novel-generator 仓库内，不拦截
        return

    # fixture 目录例外：测试 fixture 允许包含 .novel/、章节正文、.db 等
    FIXTURE_PREFIXES = (
        "spec/fixtures/",
        "contracts/cli/fixtures/",
        "tests/fixtures/",
        "test_data/",
        "examples/",
        "docs/examples/",
    )

    def is_fixture_path(path: str) -> bool:
        return any(prefix in path for prefix in FIXTURE_PREFIXES)

    # 检查 Write/Edit 操作
    if tool_name in ["Write", "Edit"]:
        file_path = event.get("tool_input", {}).get("file_path", "")

        # fixture 路径直接放行
        if is_fixture_path(file_path):
            return

        # 拦截在本仓库创建 .novel/ 目录下的文件
        if "/.novel/" in file_path or file_path.startswith(".novel/"):
            print(
                "❌ 危险操作被拦截：在 novel-generator 仓库创建 .novel/ 文件\n\n"
                "原因：CLAUDE.md 规定 novel-generator 是生成器仓库，不应保存具体小说项目的 .novel/ 数据。\n\n"
                "如何继续：\n"
                "1. 在目标小说项目目录中创建 .novel/ 文件\n"
                "2. 或者创建测试 fixture（应保存在 spec/fixtures/ 或 contracts/cli/fixtures/）",
                file=sys.stderr
            )
            sys.exit(2)

        # 拦截章节正文相关路径
        if re.search(r"(chapters?|content)/.*\.md$", file_path, re.IGNORECASE):
            # 例外：docs/、openspec/、examples/、tests/ 和 reference/ 下的 md 文件允许
            if not (file_path.startswith("docs/") or
                    file_path.startswith("openspec/") or
                    file_path.startswith("examples/") or
                    file_path.startswith("tests/") or
                    file_path.startswith("reference/")):
                print(
                    "⚠️  疑似章节正文被写入 novel-generator 仓库\n\n"
                    "检测到路径包含 'chapter' 或 'content' 且为 .md 文件。\n\n"
                    "如果这是：\n"
                    "- 具体小说章节正文 → 应写入目标小说项目的 .novel/content/chapters/\n"
                    "- 测试 fixture → 应写入 spec/fixtures/ 或 contracts/cli/fixtures/\n"
                    "- 文档或示例 → 应写入 docs/ 或 README\n\n"
                    "如果确认这是合法文件，请在 .claude/hooks/project-boundary-guard.py 中添加例外。",
                    file=sys.stderr
                )
                sys.exit(2)

    # 检查 Bash 操作
    if tool_name == "Bash":
        command = event.get("tool_input", {}).get("command", "")

        # fixture 路径直接放行
        if is_fixture_path(command):
            return

        # 拦截创建 .novel/ 目录
        if re.search(r"mkdir.*\.novel", command, re.IGNORECASE):
            print(
                "❌ 危险操作被拦截：在 novel-generator 仓库创建 .novel/ 目录\n\n"
                "原因：CLAUDE.md 规定 novel-generator 是生成器仓库，不应保存具体小说项目的 .novel/ 数据。\n\n"
                "如何继续：\n"
                "1. 在目标小说项目目录中创建 .novel/\n"
                "2. 或者通过 `novel init` 命令初始化（当 CLI 实现后）",
                file=sys.stderr
            )
            sys.exit(2)

        # 拦截 SQLite 数据库文件操作（收紧正则，只检测真正的 .db 文件操作）
        if re.search(r"(touch|echo|>|>>)\s+[^\s]*\.db(\s|$)", command) or \
           re.search(r"sqlite3?\s+[^\s]*\.db", command):
            print(
                "⚠️  检测到 SQLite 数据库文件操作\n\n"
                "novel-generator 仓库不应直接创建或修改 novel.db。\n\n"
                "如果这是：\n"
                "- 创建世界状态数据库 → 应在目标小说项目中通过 `novel init` 命令创建\n"
                "- 测试 fixture → 应在 spec/fixtures/ 或 contracts/cli/fixtures/ 中创建",
                file=sys.stderr
            )
            sys.exit(2)

if __name__ == "__main__":
    main()
