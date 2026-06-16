"""契约守卫：软层 markdown 里的 novel-cli 调用必须真实存在于硬层 argparse。

软↔硬唯一接口是 `novel-cli <子命令>`，但软层调用写在纯文本 markdown 里，写错了
不会有任何报错——这正是软硬契约漂移的根因。本测试在 CI 侧形成持续防线：

  ① 命名空间存在性：<namespace> 必须在 bin/novel-cli 的 COMMANDS 里
  ② 子命令存在性  ：<subcommand> 必须在硬层对应 parser 注册
  ③ 旗标存在性    ：每个 --flag 必须在该 subcommand 下注册

任一不匹配即非零退出，打印精确定位（文件 + 调用行 + 不匹配项）。

无第三方依赖，直接运行：python3 tests/test_contract_guard.py
"""
import argparse
import importlib.util
import os
import re
import sys


_TESTS_DIR = os.path.dirname(os.path.realpath(__file__))
_PKG = os.path.join(_TESTS_DIR, "..")
_SRC_DIR = os.path.abspath(os.path.join(_PKG, "src"))
# 软层 spec 包：repo 内为 packages/novel-spec（与 novel-lifecycle-cli 同级）
_SPEC_DIR = os.path.abspath(os.path.join(_PKG, "..", "novel-spec"))

# 与 bin/novel-cli 的 COMMANDS 保持一致（软↔硬接口表）
COMMANDS = {
    "init":           "novel/project_init/init_project.py",
    "stage":          "novel/stages/stage_manager.py",
    "finalize":       "novel/cycle_close/cycle_close_cli.py",
    "style":          "novel/style/style_cli.py",
    "mainplot":       "novel/mainplot/mainplot_cli.py",
    "world-boundary": "novel/world_boundary/world_boundary_cli.py",
    "finalcheck":     "novel/finalcheck/finalcheck_cli.py",
    "monitor":        "novel/monitor/monitor_cli.py",
    "weave":          "volume/timeline/weave_cli.py",
    "cycle":          "volume/timeline/cycle_cli.py",
    "presummary":     "volume/timeline/presummary_cli.py",
    "prose":          "chapter/prose/prose_cli.py",
    "db":             "elements/element_manager.py",
    "emergent":       "elements/element_manager.py",
    "arc":            "elements/element_manager.py",
    "bible":          "elements/element_manager.py",
    "context":        "elements/element_manager.py",
}

# 运行期二级嵌套 parser 的子命令：顶层只解析一个位置参数（如 prose residue 的
# residue_action="add"），真正的旗标在该分支内部用嵌套 parser 解析 `remaining`，
# 顶层 parser 内省看不到。对这类子命令只校验「命名空间+子命令存在性」，跳过旗标校验。
# 形如 "<namespace> <subcommand>"。新增同类二级分发命令时在此登记。
_NESTED_PARSER_SUBCOMMANDS = {
    "prose residue",
}


# ---------------------------------------------------------------------------
# 硬层 parser 内省
# ---------------------------------------------------------------------------

class _Captured(Exception):
    """在 parse_args 处截获已建好的 parser，阻止真正解析/落 DB。"""


def _load_module(rel_path):
    abs_path = os.path.abspath(os.path.join(_SRC_DIR, rel_path))
    spec = importlib.util.spec_from_file_location("_cmd_guard", abs_path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _introspect(parser):
    """从一个 argparse parser 抽出 {subcommand: set(option_strings)}。"""
    result = {}
    for action in parser._actions:
        name_map = getattr(action, "_name_parser_map", None)
        if name_map is None:
            continue
        for sub_name, sub_parser in name_map.items():
            flags = set()
            for a in sub_parser._actions:
                for opt in a.option_strings:
                    flags.add(opt)
            result[sub_name] = flags
    return result


_INTROSPECT_CACHE = {}


def _hard_layer_spec(namespace):
    """返回某命名空间下 {subcommand: set(flags)}；失败返回 None。"""
    if namespace in _INTROSPECT_CACHE:
        return _INTROSPECT_CACHE[namespace]

    if _SRC_DIR not in sys.path:
        sys.path.insert(0, _SRC_DIR)

    mod = _load_module(COMMANDS[namespace])

    captured = {}
    orig_pa = argparse.ArgumentParser.parse_args
    orig_pka = argparse.ArgumentParser.parse_known_args

    def _patched(self, args=None, namespace_=None):
        # 只截获顶层 parser（已注册子命令的那个），忽略 handler 内部用于
        # 二级解析的 parser（如 prose residue 的 p2，无 _name_parser_map）。
        if any(getattr(a, "_name_parser_map", None) for a in self._actions):
            captured["parser"] = self
            raise _Captured
        return orig_pka(self, args, namespace_)

    # 模块可能用 parse_args 或 parse_known_args（如 prose_cli）。两者都拦。
    argparse.ArgumentParser.parse_args = _patched
    argparse.ArgumentParser.parse_known_args = _patched
    try:
        # 传入 [namespace] → main 内部 parse(argv[1:]) = parse([])，
        # 截获时 parser 已注册全部子命令。element_manager 按 argv[0] 路由到子 handler。
        mod.main([namespace])
    except _Captured:
        pass
    except SystemExit:
        pass
    finally:
        argparse.ArgumentParser.parse_args = orig_pa
        argparse.ArgumentParser.parse_known_args = orig_pka

    spec = _introspect(captured["parser"]) if "parser" in captured else None
    _INTROSPECT_CACHE[namespace] = spec
    return spec


# ---------------------------------------------------------------------------
# 软层调用抽取
# ---------------------------------------------------------------------------

_CALL_RE = re.compile(r"novel-cli\s+([\w-]+)\s+([\w-]+)(.*)")
_FLAG_RE = re.compile(r"(--[\w-]+)")


def _join_continuations(text):
    """把以反斜杠续行的多行命令拼成单行。"""
    lines = text.splitlines()
    merged = []
    buf = ""
    for line in lines:
        stripped = line.rstrip()
        if stripped.endswith("\\"):
            buf += stripped[:-1] + " "
        else:
            merged.append(buf + line)
            buf = ""
    if buf:
        merged.append(buf)
    return merged


def _extract_calls(path):
    """从一个 markdown 文件抽出调用三元组列表：(namespace, subcommand, [flags], 原始行)。"""
    with open(path, encoding="utf-8") as f:
        text = f.read()
    calls = []
    for line in _join_continuations(text):
        m = _CALL_RE.search(line)
        if not m:
            continue
        namespace, subcommand, rest = m.group(1), m.group(2), m.group(3)
        flags = _FLAG_RE.findall(rest)
        calls.append((namespace, subcommand, flags, line.strip()))
    return calls


def _spec_markdown_files():
    files = []
    for sub in ("skills", "agents"):
        base = os.path.join(_SPEC_DIR, sub)
        if not os.path.isdir(base):
            continue
        for root, _dirs, names in os.walk(base):
            for name in names:
                if name.endswith(".md"):
                    files.append(os.path.join(root, name))
    return sorted(files)


# ---------------------------------------------------------------------------
# 测试
# ---------------------------------------------------------------------------

def test_contract_guard():
    files = _spec_markdown_files()
    assert files, f"未找到任何软层 markdown（查找路径：{_SPEC_DIR}）"

    errors = []
    checked = 0

    for path in files:
        rel = os.path.relpath(path, _SPEC_DIR)
        for namespace, subcommand, flags, raw in _extract_calls(path):
            checked += 1

            # ① 命名空间存在性
            if namespace not in COMMANDS:
                errors.append(
                    f"[{rel}] 未知命名空间 '{namespace}'\n        调用：{raw}"
                )
                continue

            # ② 子命令存在性
            spec = _hard_layer_spec(namespace)
            if spec is None:
                errors.append(
                    f"[{rel}] 无法内省命名空间 '{namespace}' 的 parser\n        调用：{raw}"
                )
                continue
            if subcommand not in spec:
                avail = ", ".join(sorted(spec)) or "(无)"
                errors.append(
                    f"[{rel}] 子命令不存在：{namespace} {subcommand}\n"
                    f"        硬层可用：{avail}\n        调用：{raw}"
                )
                continue

            # ③ 旗标存在性（二级嵌套 parser 的子命令只校验到 ②，旗标在运行期内层解析）
            if f"{namespace} {subcommand}" in _NESTED_PARSER_SUBCOMMANDS:
                continue
            valid_flags = spec[subcommand]
            for flag in flags:
                if flag not in valid_flags:
                    avail = ", ".join(sorted(valid_flags)) or "(无)"
                    errors.append(
                        f"[{rel}] 旗标不存在：{namespace} {subcommand} {flag}\n"
                        f"        硬层可用：{avail}\n        调用：{raw}"
                    )

    if errors:
        print(f"\n契约守卫发现 {len(errors)} 处软硬不匹配（已校验 {checked} 条调用）：\n")
        for e in errors:
            print("  ✗ " + e)
        raise AssertionError(f"{len(errors)} 处软硬契约漂移，详见上方")

    print(f"test_contract_guard PASSED（{checked} 条调用全部对齐，覆盖 {len(files)} 个文件）")


if __name__ == "__main__":
    test_contract_guard()
