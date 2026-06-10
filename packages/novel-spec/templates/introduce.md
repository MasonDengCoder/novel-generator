# 工具使用导引

## 安装

```bash
# 完整安装（软层 + 全局 novel-cli，目标项目内无 scripts/ 副本）
node /path/to/novel-generator/packages/novel-spec/install-spec.js [目标目录]
# 仅装硬层（无 LLM 环境，复制自包含 scripts/ 副本）
node /path/to/novel-generator/packages/novel-lifecycle-cli/install-cli.js [目标目录]
```

宿主机须已安装 `python3 >= 3.8`。完整安装后 `novel-cli` 在全局 PATH，于项目目录内直接调用（默认操作 CWD 下 `index/novel.db`）。

## 首次使用流程

```
/capture-inspiration   → /draft-premise          # 灵感阶段
/design-world-boundary                            # 四极设计（需委派 world-boundary-validator）
/design-style-guard                               # 文风红线
/design-mainplot                                  # 主线节点
/design-destiny                                   # 旁门设计
/weave-fate                                       # 编织结构
/write-prose                                      # 写正文（需委派 mainplot-landing-verifier）
/extract-residue                                  # 章末残余
/dispose-emergent                                 # 封卷前处置涌现
/seal-creation                                    # 终检封档
```

## 常用 CLI 查询

```bash
novel-cli bible core            # 世界核心/四极（只读）
novel-cli bible active          # 当前卷活跃状态
novel-cli bible full            # 完整圣经视图
novel-cli emergent list         # 涌现台账
novel-cli db list-all-destiny   # 旁门台账概览
novel-cli context chapter <id> # 本章线索绑定
```

## DB 路径

默认 `index/novel.db`，可用 `--db-path` 或 `NOVEL_DB` 环境变量覆盖。
