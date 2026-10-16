#!/usr/bin/env python3
"""
OpenSpec Archive Guard - 防止在验证未通过时归档 OpenSpec Change

拦截目标：
1. tasks.md 中有未完成的 checkbox
2. verify 阶段发现证据缺失
3. 实现与 specs/design 不一致

来源：config.yaml rules.archive 和 rules.verify
"""
import sys
import json
import re
import os
from pathlib import Path

def main():
    event = json.loads(sys.stdin.read())

    tool_name = event.get("tool_name")
    if tool_name != "Bash":
        return

    command = event.get("tool_input", {}).get("command", "")

    # 检测是否是 openspec archive 命令
    if not re.search(r"openspec.*archive", command, re.IGNORECASE):
        return

    # 提取 change ID（如果有）
    change_id_match = re.search(r"(?:--change-id|-c)\s+(\S+)", command)
    if not change_id_match:
        # 可能使用当前工作目录的 change
        # 简化处理：发出警告但不阻断
        print(
            "⚠️  OpenSpec Archive 警告\n\n"
            "检测到 openspec archive 命令，但未明确指定 change ID。\n\n"
            "Archive 前请确认：\n"
            "1. 所有 tasks.md checkbox 已完成\n"
            "2. 所有 verify 检查已通过\n"
            "3. 实现与 specs/design 一致\n"
            "4. 相关文档未因变更而过时\n\n"
            "如需跳过此检查，请手动确认后继续。",
            file=sys.stderr
        )
        # 警告但不阻断（因为无法定位具体 change 文件）
        return

    change_id = change_id_match.group(1)

    # 尝试查找对应的 tasks.md 和 verify.md
    openspec_dir = Path("openspec")
    changes_dir = openspec_dir / "changes"

    if not changes_dir.exists():
        # 没有 changes 目录，无法检查
        return

    # 查找 change 目录
    change_dirs = list(changes_dir.glob(f"*{change_id}*"))
    if not change_dirs:
        # 找不到对应的 change 目录
        return

    change_dir = change_dirs[0]
    tasks_file = change_dir / "tasks.md"
    verify_file = change_dir / "verify.md"

    # 检查 tasks.md
    if tasks_file.exists():
        with open(tasks_file, "r", encoding="utf-8") as f:
            tasks_content = f.read()

        # 检测未完成的 checkbox
        unchecked_boxes = re.findall(r"- \[ \] (.+)", tasks_content)
        if unchecked_boxes:
            print(
                f"❌ Archive 被拦截：tasks.md 中有 {len(unchecked_boxes)} 个未完成任务\n\n"
                "根据 config.yaml rules.archive:\n"
                "「确认所有 verify 检查已通过」\n\n"
                "未完成的任务：\n" +
                "\n".join(f"  - {task[:80]}" for task in unchecked_boxes[:5]) +
                (f"\n  ... 还有 {len(unchecked_boxes) - 5} 个" if len(unchecked_boxes) > 5 else "") +
                "\n\n如何继续：\n"
                "1. 完成所有 tasks.md 中的任务\n"
                "2. 或者更新 tasks.md 删除不需要的任务\n"
                "3. 运行 /openspec-verify-change 确认完成状态",
                file=sys.stderr
            )
            sys.exit(2)

    # 检查 verify.md
    if verify_file.exists():
        with open(verify_file, "r", encoding="utf-8") as f:
            verify_content = f.read()

        # 检测未验证的项目
        if re.search(r"❌|BLOCKED|FAILED|证据缺失|未完成", verify_content, re.IGNORECASE):
            print(
                "❌ Archive 被拦截：verify.md 中发现未通过的验证\n\n"
                "根据 config.yaml rules.archive:\n"
                "「确认所有 verify 检查已通过」\n\n"
                "如何继续：\n"
                "1. 运行 /openspec-verify-change 重新验证\n"
                "2. 修复所有 BLOCKED 或 FAILED 的检查项\n"
                "3. 补充缺失的证据（命令输出、测试结果、文件存在性）",
                file=sys.stderr
            )
            sys.exit(2)

        # 检测缺失必要标记（如 "✅ 所有检查通过" 等）
        if not re.search(r"✅.*通过|✅.*完成|验证通过", verify_content):
            print(
                "⚠️  Archive 警告：verify.md 中未找到明确的验证通过标记\n\n"
                "根据 config.yaml rules.verify:\n"
                "「证据缺失视为未完成」\n\n"
                "建议：\n"
                "1. 运行 /openspec-verify-change 生成完整验证报告\n"
                "2. 确认 verify.md 中包含 「✅ 所有检查通过」 标记",
                file=sys.stderr
            )
            # 警告但不阻断（可能是格式不同）
            return

    # 生成物漂移闸：若仓库已含 contracts/cli/generated/（即有 @codegen spec），
    # archive 前必须 scripts/check-generated-drift.sh 通过。
    generated_dir = Path("contracts/cli/generated")
    if generated_dir.exists() and any(generated_dir.iterdir()):
        drift_script = Path("scripts/check-generated-drift.sh")
        if drift_script.exists():
            import subprocess
            result = subprocess.run(
                ["bash", str(drift_script)],
                capture_output=True, text=True
            )
            if result.returncode != 0:
                print(
                    "❌ Archive 被拦截：生成物漂移检查未通过\n\n"
                    "根据 config.yaml rules.archive:\n"
                    "「涉及含 @codegen 块的 spec 时，归档前 scripts/check-generated-drift.sh 必须通过」\n\n"
                    "如何继续：\n"
                    "1. 运行 `cargo run -p spec-gen -- generate` 重新生成契约测试\n"
                    "2. 再次运行 `scripts/check-generated-drift.sh` 确认无漂移\n"
                    "3. 修复 SSOT 对齐闸或 fixture 引用闸报告的问题",
                    file=sys.stderr
                )
                sys.exit(2)

if __name__ == "__main__":
    main()
