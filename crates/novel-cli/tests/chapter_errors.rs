// 验证 chapter 错误码映射正确（task 3.4）。

use novel_cli::errors::CliError;

#[test]
fn chapter_not_found_maps_to_exit_code_20() {
    let err = CliError::chapter_not_found("Chapter chapter-999 not found", None);
    assert_eq!(err.exit_code() as i32, 20);
    assert_eq!(err.code().as_str(), "chapter_not_found");
}

#[test]
fn content_modified_externally_maps_to_exit_code_21() {
    let err = CliError::content_modified_externally("Content hash mismatch detected");
    assert_eq!(err.exit_code() as i32, 21);
    assert_eq!(err.code().as_str(), "content_modified_externally");
}

#[test]
fn file_not_found_maps_to_exit_code_11() {
    let err = CliError::file_not_found("Input file not found");
    assert_eq!(err.exit_code() as i32, 11);
    assert_eq!(err.code().as_str(), "file_not_found");
}

#[test]
fn invalid_content_maps_to_exit_code_2() {
    let err = CliError::invalid_content("File is not valid UTF-8");
    assert_eq!(err.exit_code() as i32, 2);
    assert_eq!(err.code().as_str(), "invalid_content");
}

#[test]
fn chapter_errors_have_suggestions() {
    let err = CliError::chapter_not_found("Chapter not found", Some("使用 `novel chapter list` 查看所有章节。".to_string()));
    assert!(err.suggestion().is_some());
    assert!(err.suggestion().unwrap().contains("novel chapter list"));

    let err = CliError::content_modified_externally("Hash mismatch");
    assert!(err.suggestion().is_some());
    assert!(err.suggestion().unwrap().contains("novel chapter reimport"));
}
