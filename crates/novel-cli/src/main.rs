// novel CLI 入口。
//
// 职责（CLAUDE.md §6）：只启动 CLI parser 与顶层异常捕获/退出码处理。
// 不直接访问 storage / repository，不拼接业务 JSON。
//
// bin -> command parser -> command adapter -> service/use-case
//      -> validator/guard -> repository -> storage transaction -> renderer

fn main() {
    let exit_code = novel_cli::run();
    std::process::exit(exit_code);
}
