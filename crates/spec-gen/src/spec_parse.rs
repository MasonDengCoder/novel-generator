//! spec 解析：扫 openspec/specs/**/*.md，从每个 #### Scenario: 块定位紧随其后的
//! `<!-- @codegen ... -->` HTML 注释，serde_yaml 反序列化为 CodegenBlock。
//!
//! 试点阶段只有 chapter-context 带 @codegen 块；不带块的 scenario 跳过（不报错）。

use serde::Deserialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 一个 @codegen 块解析结果（原 spec 文件名 + YAML 块内容）。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenBlock {
    /// 测试函数名（snake_case，唯一）。
    pub id: String,
    /// fixture 脚本名（contracts/cli/fixtures/<name>.sh，不带路径与扩展名之外的部分）。
    pub fixture: String,
    /// 被测 CLI 命令完整串（novel ... --json）。
    pub when: String,
    /// 断言项列表：每项是单键 mapping，键决定断言类型。
    pub assert: Vec<serde_yaml::Mapping>,
}

/// 解析结果块：来源 spec 文件 + CodegenBlock。
#[derive(Debug, Clone)]
pub struct ParsedBlock {
    /// spec 相对路径（openspec/specs/chapter-context/spec.md）。
    pub spec_rel: String,
    /// domain（spec 目录名，chapter-context）。
    pub domain: String,
    /// scenario 标题（H4 行去掉前缀），仅用于注释/报告，不作为稳定 ID。
    pub scenario_title: String,
    /// @codegen 块内容。
    pub block: CodegenBlock,
}

/// 扫描 specs_dir，返回全部带 @codegen 块的 ParsedBlock。
/// fixture 引用闸：block.fixture 必须指向 fixtures_dir 下真实存在的 <name>.sh。
pub fn parse_specs(specs_dir: &Path, fixtures_dir: &Path) -> anyhow::Result<Vec<ParsedBlock>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(specs_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let text = std::fs::read_to_string(path)?;
        let spec_rel = path.strip_prefix(specs_dir.parent().unwrap_or(specs_dir))
            .unwrap_or(path).to_string_lossy().replace('\\', "/");
        let domain = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        for block in parse_one(&text, &domain, &spec_rel, fixtures_dir)? {
            out.push(block);
        }
    }
    // 去重：同 id 出现多次报错（id 是稳定唯一键）。
    let mut seen = std::collections::HashSet::new();
    for b in &out {
        if !seen.insert(&b.block.id) {
            anyhow::bail!("@codegen id '{}' 在多个 spec 重复（id 必须全局唯一）", b.block.id);
        }
    }
    Ok(out)
}

/// 解析单个 spec 文件的全部 @codegen 块。
fn parse_one(
    text: &str,
    domain: &str,
    spec_rel: &str,
    fixtures_dir: &Path,
) -> anyhow::Result<Vec<ParsedBlock>> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut current_title = String::new(); // 当前所在 scenario 标题
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        let line = raw.trim();
        // scenario 边界：#### Scenario: <title>
        if let Some(title) = line.strip_prefix("#### Scenario:") {
            current_title = title.trim().to_string();
            i += 1;
            continue;
        }
        // requirement 边界：清空当前 scenario 上下文（块不可跨 requirement）。
        if line.starts_with("### Requirement:") {
            current_title.clear();
            i += 1;
            continue;
        }
        // @codegen 注释块起点。
        if line.starts_with("<!-- @codegen") {
            // 收集注释块直到 -->
            let mut yaml = String::new();
            let mut consumed = 1; // 至少消费起点行
            if line.contains("-->") && line.len() > "<!-- @codegen".len() {
                // 单行块形如 <!-- @codegen ... -->
                let inner = line.trim_start_matches("<!-- @codegen");
                let inner = inner.trim_end_matches("-->");
                yaml = inner.trim_start_matches('\n').to_string();
            } else {
                for k in (i + 1)..lines.len() {
                    consumed = k - i + 1;
                    let lk = lines[k];
                    if lk.trim().starts_with("-->") {
                        break;
                    }
                    yaml.push_str(lk);
                    yaml.push('\n');
                    if lk.trim() == "-->" || lk.trim().ends_with("-->") {
                        break;
                    }
                }
            }
            if current_title.is_empty() {
                anyhow::bail!("spec {spec_rel}: @codegen 块出现在 #### Scenario: 之外（行 {}）", i + 1);
            }
            let block: CodegenBlock = serde_yaml::from_str(&yaml).map_err(|e| {
                anyhow::anyhow!("spec {spec_rel} scenario '{current_title}' @codegen 块解析失败: {e}\n--- 块内容 ---\n{yaml}")
            })?;
            // fixture 引用闸
            let fixture_path = fixtures_dir.join(format!("{}.sh", block.fixture));
            if !fixture_path.exists() {
                anyhow::bail!(
                    "[fixture 闸] @codegen id='{}' 引用 fixture '{}'，但 {} 不存在",
                    block.id, block.fixture, fixture_path.display()
                );
            }
            blocks.push(ParsedBlock {
                spec_rel: spec_rel.to_string(),
                domain: domain.to_string(),
                scenario_title: current_title.clone(),
                block,
            });
            i += consumed;
            continue;
        }
        i += 1;
    }
    Ok(blocks)
}
