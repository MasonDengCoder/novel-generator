# 工具使用导引

## 安装

```bash
# 完整安装（软层 + 硬层）
node /path/to/novel-generator/packages/novel-spec/install-spec.js [目标目录]
# 仅装硬层
node /path/to/novel-generator/packages/novel-lifecycle-cli/install-cli.js [目标目录]
```

宿主机须已安装 `python3 >= 3.8`。

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
scripts/novel-cli bible core            # 世界核心/四极（只读）
scripts/novel-cli bible active          # 当前卷活跃状态
scripts/novel-cli bible full            # 完整圣经视图
scripts/novel-cli emergent list         # 涌现台账
scripts/novel-cli db list-all-destiny   # 旁门台账概览
scripts/novel-cli context chapter <id> # 本章线索绑定
```

## DB 路径

默认 `index/novel.db`，可用 `--db-path` 或 `NOVEL_DB` 环境变量覆盖。
