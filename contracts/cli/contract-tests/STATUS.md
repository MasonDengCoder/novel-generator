# Contract Tests Status

> 本文件记录各契约测试的当前状态：通过/失败/待实现。随着切片实现推进，及时更新状态。

## 测试状态

| 测试文件 | 覆盖命令 | 状态 | 备注 |
|---|---|---|---|
| `e2e.rs` | `novel init`、`novel info`、`novel chapter status`（全链路） | ✅ **通过** | `cli-spine-init-info` + `s04-state-machine-and-verdicts` 切片已实现并通过测试；新增章节生命周期全链路场景 |
| `setting_commands.rs` | `novel setting set/get/list` | ✅ **通过** | `s01-project-bootstrap` 切片已实现并通过测试（当前 `schemaVersion` 见 golden 与 `CURRENT_SCHEMA_VERSION`） |
| `constraint_commands.rs` | `novel constraint define/render/list` | ✅ **通过** | `s01-project-bootstrap` 切片已实现并通过测试（当前 `schemaVersion` 见 golden 与 `CURRENT_SCHEMA_VERSION`） |
| `verdict.rs` | `novel verdict set-mainplot-landing` | ✅ **通过** | `s04-state-machine-and-verdicts` 切片已实现（9 个测试：写入成功/覆盖/外部修改/章节不存在/参数错误/approved 拒绝/未初始化） |
| `state_transition.rs` | `novel chapter status` | ✅ **通过** | `s04-state-machine-and-verdicts` 切片已实现（13 个测试：前向推进/非法路径/终态/hard 违规/裁决未达成/stale/外部修改/回退/多项失败/**approved 拒绝 update 正文**） |
| `volume_commands.rs` | `novel volume create/info/list/seal/archive/goal` | ✅ **通过** | `s05-volume-management` + `s07-archive-and-export`：封卷真实检查、sealing 中间态、归档冻结（s07 新增 6 个测试） |
| `export_commands.rs` | `novel export volume/chapter/full` | ✅ **通过** | `s07-archive-and-export`：导出归档卷/未封卷含 warning/卷不存在/章节不存在/hash 失配/JSON 纯净性（6 个测试） |
| `verify_commands.rs` | `novel verify integrity` | ✅ **通过** | `s07-archive-and-export`：全通过/hash 失配/悬空引用/stale 裁决/未初始化/人读摘要（6 个测试） |
| `archive_export_e2e.rs`（位于 `crates/novel-cli/tests/e2e/`，非契约测试目录） | s07 全链路 e2e | ✅ **通过** | `s07-archive-and-export`：init→volume→chapter→seal→archive→export→verify（2 个测试） |
| `mainplot_contract_test.rs` | `novel mainplot node create/list/get` | ✅ **通过** | `s06-destiny-weaving`：JSON schema、ID 格式 `mp-<vol>-<seq>`、not_found 错误对象（5 个测试） |
| `thread_contract_test.rs` | `novel thread create/bind/list/query` | ✅ **通过** | `s06-destiny-weaving`：JSON schema、name_conflict/not_found/binding_already_exists 错误对象（5 个测试） |
| `emergence_contract_test.rs` | `novel emergence create/rate/decide/execute/archive/list/get` | ✅ **通过** | `s06-destiny-weaving`：JSON schema、invalid_state_transition/not_found/archived 错误对象、生命周期字段演化（6 个测试） |
| `chapter_context_commands.rs` | `novel chapter context --view` | ✅ **通过** | `chapter-context-packet` 切片（2026-07-22）：writer/verifier/repair packet 结构、budget-exceeded（exit 3 `context_budget_exceeded` + partialPacket）、invalid/missing view（exit 2）、chapter_not_found（exit 20）、人读摘要 |
| `chapter-context.gen.rs`（生成式，位于 `contracts/cli/generated/`） | `novel chapter context --view` | ✅ **通过** | 从 `openspec/specs/chapter-context/spec.md` 的 `@codegen` 块生成（spec-gen xtask）：15 个测试覆盖三视图/P0P1 装填/诚实省略/budget 超限/invalid view/missing budget/chapter_not_found/world_uninit/hash mismatch |
| `chapter-lifecycle.gen.rs`（生成式，位于 `contracts/cli/generated/`） | `novel chapter create --volume` | ✅ **通过** | 从 `openspec/specs/chapter-lifecycle/spec.md` 的 `@codegen` 块生成（spec-gen xtask）：6 个测试覆盖 with/without volume/volume_not_found/volume_not_active/world_uninit/non-integer |

## 状态说明

- ✅ **通过**：命令已实现，契约测试全部通过，契约面已固化，Spec 可依赖。
- ⏳ **待实现**：契约面已定义（commands.md、json-schemas、golden examples），测试已编写，但命令实现尚未完成。
- ❌ **失败**：命令已实现但契约测试失败，需修复实现或更新契约。

## 当前测试运行结果

### `e2e.rs` - ✅ 全部通过

```bash
$ cargo test -p novel-cli --test e2e
running 3 tests
test e2e_init_then_info_full_chain ... ok
test e2e_init_then_info_separate_directories_independent ... ok
test e2e_chapter_lifecycle_create_to_approved ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `setting_commands.rs` - ✅ 全部通过

```bash
$ cargo test -p novel-cli --test setting_commands
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `constraint_commands.rs` - ✅ 全部通过

```bash
$ cargo test -p novel-cli --test constraint_commands
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `verdict.rs` - ✅ 全部通过（s04-state-machine-and-verdicts）

```bash
$ cargo test -p novel-cli --test verdict
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `state_transition.rs` - ✅ 全部通过（s04-state-machine-and-verdicts）

```bash
$ cargo test -p novel-cli --test state_transition
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `volume_commands.rs` - ✅ 全部通过（s05 + s07）

```bash
$ cargo test -p novel-cli --test volume_commands
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `export_commands.rs` - ✅ 全部通过（s07-archive-and-export）

```bash
$ cargo test -p novel-cli --test export_commands
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `verify_commands.rs` - ✅ 全部通过（s07-archive-and-export）

```bash
$ cargo test -p novel-cli --test verify_commands
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `archive_export_e2e.rs`（位于 `crates/novel-cli/tests/e2e/`） - ✅ 全部通过（s07-archive-and-export）

```bash
$ cargo test -p novel-cli --test archive_export_e2e
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `mainplot_contract_test.rs` - ✅ 全部通过（s06-destiny-weaving）

```bash
$ cargo test -p novel-cli --test mainplot_contract_test
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `thread_contract_test.rs` - ✅ 全部通过（s06-destiny-weaving）

```bash
$ cargo test -p novel-cli --test thread_contract_test
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `emergence_contract_test.rs` - ✅ 全部通过（s06-destiny-weaving）

```bash
$ cargo test -p novel-cli --test emergence_contract_test
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 归档切片前的验收条件

归档 `s04-state-machine-and-verdicts` 切片时，必须确保：

1. `verdict.rs` 全部 10 个测试通过。
2. `state_transition.rs` 全部 13 个测试通过。
3. `e2e.rs` 章节生命周期全链路场景通过。
4. `exit-codes.md` 为 `novel chapter status` 与 `novel verdict set-mainplot-landing` 增退出码场景表（复用 0/2/3/20/21/70）。
5. `error-codes.md` 在命令场景下复用既有错误码，不新增 §A/§B 条目。
6. 所有 JSON 输出符合对应的 JSON Schema（`verdict.schema.json`、更新后的 `chapter.schema.json` 含 `status`）。

## 下次更新时机

- **归档 `s04-state-machine-and-verdicts` 切片时**：确认所有契约测试通过后，归档该切片。
- **后续切片扩展裁决类型时**：在 `verdicts.md` 与 `verdict.schema.json` 中扩展 `verdictType`，新增对应命令与测试。
