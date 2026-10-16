// 契约测试：验证外部修改检测返回正确退出码 21（task 10.5）。

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_chapter_hash_mismatch_on_get() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目并创建章节
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .assert()
        .success();

    // 写入正文
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "原始内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 验证可以成功读取
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .assert()
        .success();

    // 外部修改正文文件 - content_path 是从 project_dir 开始的相对路径
    let content_path = project_dir.join(".novel/content/chapters/chapter-001.md");

    // 确保文件存在后再修改
    if !content_path.exists() {
        // 尝试找到文件
        let content_dir = project_dir.join(".novel/content");
        eprintln!("Looking for file at: {}", content_path.display());
        eprintln!(".novel/content exists: {}", content_dir.exists());
        if content_dir.exists() {
            eprintln!(".novel/content contents: {:?}", fs::read_dir(&content_dir).unwrap().map(|e| e.unwrap().path()).collect::<Vec<_>>());
            let chapters_dir = content_dir.join("chapters");
            if chapters_dir.exists() {
                eprintln!(".novel/content/chapters contents: {:?}", fs::read_dir(&chapters_dir).unwrap().map(|e| e.unwrap().path()).collect::<Vec<_>>());
            } else {
                eprintln!(".novel/content/chapters does not exist");
            }
        }

        // 列出 .novel/.novel 目录（这个看起来奇怪）
        let weird_novel = project_dir.join(".novel/.novel");
        if weird_novel.exists() {
            eprintln!(".novel/.novel exists and contains: {:?}", fs::read_dir(&weird_novel).unwrap().map(|e| e.unwrap().path()).collect::<Vec<_>>());
        }

        panic!("正文文件不存在：{}", content_path.display());
    }

    fs::write(&content_path, "这是完全不同的外部修改内容，应该触发 hash mismatch").unwrap();

    // 验证文件确实被修改了
    let modified_content = fs::read_to_string(&content_path).unwrap();
    assert!(modified_content.contains("外部修改"), "文件应已被修改");

    // 读取章节应检测到外部修改
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .assert()
        .failure()
        .code(5); // E_CONTENT_MODIFIED_EXTERNALLY → external_modification 类别退出码
}

#[test]
fn test_chapter_hash_mismatch_on_list_verify() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目并创建章节
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .assert()
        .success();

    // 写入正文
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "原始内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 外部修改正文文件
    let novel_dir = project_dir.join(".novel");
    let content_path = novel_dir.join("content/chapters/chapter-001.md");
    fs::write(&content_path, "外部修改").unwrap();

    // 列出章节并验证 hash 应检测到外部修改
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("list")
        .arg("--verify-hash")
        .assert()
        .failure()
        .code(5); // E_CONTENT_MODIFIED_EXTERNALLY → external_modification 类别退出码
}

#[test]
fn test_chapter_reimport_after_external_modification() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目并创建章节
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .assert()
        .success();

    // 写入正文
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "原始内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 外部修改正文文件
    let novel_dir = project_dir.join(".novel");
    let content_path = novel_dir.join("content/chapters/chapter-001.md");
    fs::write(&content_path, "外部修改").unwrap();

    // 重新导入
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("reimport")
        .arg("chapter-001")
        .assert()
        .success();

    // 现在读取应成功
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .assert()
        .success();
}
