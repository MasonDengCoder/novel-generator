// chapter_validator 模块：章节标题、正文文件、word_count、hash 校验（task 4.1-4.4）。

use crate::errors::CliError;
use sha2::{Sha256, Digest};
use std::path::Path;

/// 校验章节标题：非空、长度限制。
pub fn validate_chapter_title(title: &str) -> Result<(), CliError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(CliError::usage("章节标题不能为空。"));
    }
    if trimmed.len() > 200 {
        return Err(CliError::usage(format!(
            "章节标题过长（{}字符），最大允许 200 字符。",
            trimmed.len()
        )));
    }
    Ok(())
}

/// 校验正文文件：存在性、UTF-8 编码、文件大小限制。
pub fn validate_content_file(file_path: &Path) -> Result<String, CliError> {
    // 检查文件存在性
    if !file_path.exists() {
        return Err(CliError::file_not_found(format!(
            "输入文件不存在：{}",
            file_path.display()
        )));
    }

    // 检查是否为文件
    if !file_path.is_file() {
        return Err(CliError::usage(format!(
            "路径不是文件：{}",
            file_path.display()
        )));
    }

    // 读取文件内容
    let content = std::fs::read_to_string(file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            CliError::invalid_content(format!(
                "文件不是有效的 UTF-8 编码：{}",
                file_path.display()
            ))
        } else {
            CliError::storage(format!(
                "读取文件失败：{} - {}",
                file_path.display(),
                e
            ))
        }
    })?;

    // 检查文件大小限制（10MB）
    if content.len() > 10 * 1024 * 1024 {
        return Err(CliError::usage(format!(
            "文件过大（{}字节），最大允许 10MB。",
            content.len()
        )));
    }

    Ok(content)
}

/// 计算 word_count：去除 Markdown 标记，中文按字符计数，英文按单词计数。
pub fn calculate_word_count(content: &str) -> usize {
    // 简化实现：去除 Markdown 标题、代码块标记
    let text = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter(|line| !line.trim().starts_with("```"))
        .collect::<Vec<_>>()
        .join(" ");

    // 统计：中文字符 + 英文单词
    let mut count = 0;
    let mut in_word = false;

    for ch in text.chars() {
        if ch >= '\u{4e00}' && ch <= '\u{9fff}' {
            // CJK 统一汉字范围 - 每个汉字算一个字
            count += 1;
            in_word = false;
        } else if ch.is_ascii_alphanumeric() {
            if !in_word {
                count += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }

    count
}

/// 计算 SHA-256 hash（十六进制编码）。
pub fn calculate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_title_ok() {
        assert!(validate_chapter_title("第一章").is_ok());
        assert!(validate_chapter_title("Chapter 1: The Beginning").is_ok());
    }

    #[test]
    fn test_validate_title_empty() {
        assert!(validate_chapter_title("").is_err());
        assert!(validate_chapter_title("   ").is_err());
    }

    #[test]
    fn test_validate_title_too_long() {
        let long_title = "a".repeat(201);
        assert!(validate_chapter_title(&long_title).is_err());
    }

    #[test]
    fn test_calculate_word_count_chinese() {
        assert_eq!(calculate_word_count("这是一个测试"), 6);
        assert_eq!(calculate_word_count("第一章：起源"), 5); // 冒号不计入
    }

    #[test]
    fn test_calculate_word_count_english() {
        assert_eq!(calculate_word_count("This is a test"), 4);
        assert_eq!(calculate_word_count("Hello world"), 2);
    }

    #[test]
    fn test_calculate_word_count_mixed() {
        assert_eq!(calculate_word_count("这是 a test 测试"), 6); // 2汉字 + 2英文单词 + 2汉字
    }

    #[test]
    fn test_calculate_word_count_with_markdown() {
        assert_eq!(calculate_word_count("# 标题\n这是正文"), 4);
        assert_eq!(calculate_word_count("```code```\n文本"), 2);
    }

    #[test]
    fn test_calculate_hash_deterministic() {
        let content = "测试内容";
        let hash1 = calculate_content_hash(content);
        let hash2 = calculate_content_hash(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_calculate_hash_different() {
        let hash1 = calculate_content_hash("内容A");
        let hash2 = calculate_content_hash("内容B");
        assert_ne!(hash1, hash2);
    }
}
