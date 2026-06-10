# 小说创作工具包

本项目使用 `novel-generator` 工具包，运行在 Claude Code 上。

## 铁律（任何阶段均适用）

1. 与硬层交互只允许通过 `novel-cli <子命令>`，禁止绕过 CLI 直接读写 DB
2. 正文不入库：DB 只存 `prose_ref`（弱路径）和 `word_count` 遥测
3. 不可越级修正：遁出/改主线须走四级阶梯（章标记→事件轻修→卷处置→周期归档）
4. 所有元素读写经 `element_manager.py` 唯一入口，不直接拼 SQL
5. 世界圣经（Bible）是 DB 的派生视图，禁止独立持久化
6. Hook 等工具异常时静默降级，绝不因工具故障中断写作

## 创作阶段

| 阶段 | Skill | 说明 |
|------|-------|------|
| 灵感 | `/capture-inspiration` → `/draft-premise` | 奇点 + 一句话前提 |
| 四极 | `/design-world-boundary` | ≥3 硬边界 + 1 模糊边界 |
| 文风 | `/design-style-guard` | 文笔风格定义 + 敏感词红线 |
| 主线 | `/design-mainplot` | 主线节点（标注 immutable 定数） |
| 旁门 | `/design-destiny` | 支线/伏笔/爽点/承诺 |
| 编织 | `/weave-fate` | 卷→事件→小节→章层级结构 |
| 写作 | `/write-prose` | 正文写作（写验分离） |
| 残余 | `/extract-residue` | 章末残余提取 |
| 封卷 | `/dispose-emergent` | 涌现处置（close-volume 前置） |
| 封印 | `/seal-creation` | 终检封档 |

详细使用说明见 `introduce.md`。
