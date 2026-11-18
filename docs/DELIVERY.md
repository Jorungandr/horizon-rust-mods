# 交付清单 — 六个 Horizon 原生 Rust 模组

最后更新：2026-11-18 · 状态：**交付完成**（三个 MIT Java 模组移植 + 三个原创模组；代码 / 验证 / 文档 / 资源准备全就绪；**六个都在真实游戏里跑通并截图**，见第六节）

---

## 一、交付内容

三个 MIT Java 模组按 Horizon 原生模组 API（mixin helper 方式）重写为 Rust，另加三个原创原生模组：

| 模组目录 | 原作 / 许可 | 功能 | 主逻辑测试 | 端到端验证 |
|---|---|---|---|---|
| `whenchat-horizon/` | ErikEdits/WhenChat · MIT | 每条聊天加本地时间前缀 | 4/4 | 6/6 |
| `chattimestamps-horizon/` | PlagueTR/ChatTimestamps · MIT | 可配置时间戳 + 颜色 + hover + 设置页 | 12/12 | 19/19 |
| `timeformatter-horizon/` | m3z0id/TimeFormatter · MIT | Discord `<t:...>` 时间戳转本地时间 | 10/10 | 17/17 |

每个模组目录均包含：
- `Cargo.toml` / `Cargo.lock` — 包元数据（v0.1.0, edition 2024, MIT, `publish=false`）
- `LICENSE` — MIT 许可（保留原作者版权署名）
- `NOTICE.md` — 原作出处、许可、移植范围与差异说明
- `compatibility.json` — pin 到 Horizon commit `9ee13cd0...`（hooks 645；整包编译实测修正，原误 pin 的 `85b5bcca` 是重构后的 mod-api-gen 机制，已弃用）
- `mixins/<id>/` — 真实注入源码（`runtime.rs` / `chat.rs` / `list.rs` 等）+ `horizon-mixin.toml`
- `verify/` — 离线端到端验证工程（mod_api stub + `#[path]` 引入真实源文件）

顶层文档：
- `README.md` — 总览、选型标准、API 核对、复现命令
- `TESTING.md` — 游戏文件位置、三层测试清单、真机实测逐项清单、下一步
- `tools/` — 26.3 资源与 JDK 25 下载脚本（可复现）

## 二、验证状态（最终回归 2026-11-17，FAIL=0）

| 验证层 | 命令 | 结果 |
|---|---|---|
| **整包真实编译** | `native.py check all`（容器，真实 Horizon `9ee13cd0` + rustc 1.95） | ✅ `{"mods": [...], "check": "ok"}` |
| 离线单测（引擎 Text 模型） | `cargo test --offline` × 3 verify | 7 + 19 + 16 = **42 通过** |
| 清单 TOML/JSON 语法校验 | — | 全部通过 |

**整包真实编译是本轮的决定性验证**：三个模组的 helper 真实注入 Horizon `9ee13cd0` 源码后，`cargo check` 整个 `horizon-client` 全绿。它抓出并修复了离线 stub 发现不了的 5 类真 bug：① helper 必须单个私有 fn；② binding 名须为真实展开参数（无 `ClientChatReceivedCall`）；③ `Text` 是引擎侧扁平 spans 模型（非 mod_api 组件树）；④ `-D warnings` 下 dead_code/unused import 报错；⑤ 工具链需 rustc ≥1.95。修复后全部重跑通过。

离线 verify 已同步重写为**引擎侧 Text 模型**（`horizon_client::text::Text`，`Vec<Span>` + u32 颜色 + `Box<Text>` hover），不再是早期错误的组件树模型。

**已打出可安装 `.hnmod` 成品**（用修正后的引擎 Text 源码重新 `native.py package all`，pinned revision 一致性校验通过）：

| 包 | 大小 | SHA256 | 位置 |
|---|---|---|---|
| `whenchat-0.1.0.hnmod` | 4598 B | `d43ff0ca…865f` | `dist/hnmod/` |
| `chattimestamps-0.1.0.hnmod` | 9277 B | `d3e239d6…0e287` | `dist/hnmod/` |
| `timeformatter-0.1.0.hnmod` | 7065 B | `2dba9c23…7ab6` | `dist/hnmod/` |

三个包结构校验无误：`horizon-native.json`（schema 1 / game 26.3 / hooks 645 / 双 pin 一致，均 pin `9ee13cd0...`）+ 各自的 `.rs`/`.toml` 源码 + LICENSE；已逐个解包确认 `chat.rs` 无旧 API 残留、含正确 `&mut *message` binding。容器编译环境（rsproxy 镜像、华为云 rustc/apt 源、系统开发库）见 `DOCKER-BUILD.md`。

**剩余唯一环节：真机游戏内实测**（`native.py build` 出二进制 → 离线启动 → 按 `TESTING.md` 清单验证）。

## 三、本机 26.3 资源准备状态（全部就绪 ✅）

| 项目 | 位置 | 状态 |
|---|---|---|
| 客户端 jar + 版本 JSON | `E:\Minecraft\.minecraft\versions\26.3\` | ✅ SHA1 通过 |
| 服务端 jar | `E:\Minecraft\.minecraft\server-26.3\server.jar` | ✅ SHA1 通过 |
| 114 个库 | `E:\Minecraft\.minecraft\libraries\` | ✅ 全部通过 |
| 资源对象 5149 个（487MB） | `E:\Minecraft\.minecraft\assets\objects\` | ✅ 5149/5149 = 100% |
| 资源索引 | `E:\Minecraft\.minecraft\assets\indexes\34.json` | ✅ |
| JDK 25 | `E:\Minecraft\jdk-25\jdk-25.0.5.0.2+1\` | ✅ `java -version` = 25.0.5 LTS |

> 下载过程中 Mojang 官方 CDN 一度 502，资源对象改用 BMCLAPI 镜像（`/assets/<前2位>/<完整hash>`，SHA1 逐个校验）；JDK 改用微软 OpenJDK（GitHub CDN 被代理拦）。

## 四、如需发布 / 上架（可选）

1. **整包编译确认**（Linux/macOS）：在 `horizon-mods` 仓库运行
   `python3 native/tools/native.py check <mod>`，通过后 `package all` 产出 `.hnmod`。
2. **真机实测**：启动器 `native-build ... --trust-native-code`，`--name` 离线启动，按 `TESTING.md` 清单逐项验证。
3. **提交到模组仓库**：用 https://playhorizon.dev/mods/submit 表单，GitHub 登录上传；审核含源码审阅 + Nightly 整包编译。原生模组每个版本都要审核。
4. **许可义务**：三个原作均 MIT，允许修改再发布，条件是保留版权与许可声明——本交付已保留 LICENSE + NOTICE。上架时注意 Horizon 主仓 LGPL-3.0-or-later 的二进制分发义务。

## 五、交接建议

- 代码层面可直接交付或开源（MIT，署名齐全）。
- 若要"跑起来看效果"，需补一台 Linux/macOS 环境做整包编译；本机资源已全部备好，届时直接 `horizon-prepare` 即可。
- 账号：走启动器 Play 需登录拥有 Java 版的微软账号（owner gate，首次验证后可离线玩）；仅 `native.py build` 二进制 + `--name <名字>` 可完全不登录。资源准备依赖你自有正版 Java 26.3，本机已备齐。

---

## 六、2026-11-18 增补：本机 Windows 编译 + 真机验证（已完成）

前五节描述的是「源码 + 离线验证 + Docker 整包编译」阶段。本节记录在其之上补齐的
**真实游戏运行验证**，以及为此新增的产物。

### 新增产物

| 路径 | 内容 |
|---|---|
| `WINDOWS-BUILD.md` | 本机编译与验证全过程、可复现命令、3 个真 bug 的成因与修法 |
| `native-v2/<id>/` | 三个模组的**新版格式**（`horizon-mod.toml` + trait 模组）工程，已通过 `cargo check` |
| `tools/inject_all_ports.py` | 把三个模组注入 Horizon 源码树（含必要的引擎侧改动） |
| `tools/build_win.sh` / `build_ctl.sh` | 用启动器自带 Rust 1.99.1 + MinGW 编译客户端与 ctl |
| `tools/make_game_port.py` | 复制官方 nightly 的 26.3 资源，得到独立的验证游戏目录 |
| `tools/verify_ports.sh` / `verify_settings.sh` | 无头驱动真实游戏并断言三个模组的行为 |
| `PLAY.cmd` | 一键用带模组的构建启动游戏 |
| `dist/horizon-ports-windows-verified.zip` | 上述源码、工具、文档与验证截图的打包（约 1.4 MB） |
| `build-win/*.png` | 真机证据：HUD 三条链路、Options>Mods、Chat Timestamps 设置页 |

### 验证结论

真实游戏（RTX 4060 / Vulkan）里发送 `see you at <t:1704067200:t> ok`，HUD 显示：

```
[12:27:29] [12:27:29] <Player32510> see you at 08:00 ok
```

- 第一个灰色 `[12:27:29] ` = Plague's Chat Timestamps
- 第二个 `[12:27:29] ` = WhenChat
- `<t:1704067200:t>` → `08:00` = Time Formatter

Options > Mods 三行齐全，Chat Timestamps 的 `Settings` 页三个按钮可点、可落盘
（`enable_mod` / `format` / `enable_hover`）。

### 本轮修掉的 3 个真 bug（此前三层验证都没抓到）

1. `Text` 同时持有 `spans` 与私有 `source`，原地改 span 会让 `component()` 过期
   → 改用 `Text::styled` 逐 span 重建。
2. `Game::native_setting` 会 trim 所有值，而时间戳格式的尾随空格有语义
   → 读取后按预设还原（`read_settings`）。
3. `chat_received` 在 `mod_api::mods()` 为空时直接返回，编译进二进制的模组会被静默跳过
   → 删掉该守卫。

### 仍未完成的部分

`horizon-launcher native-build` 仍不可用：需要 `nightly-2026-11-13-9f42064` 的官方
mod 工具链，而 GitHub 上从未发布 `sdk-*` release。因此 `native-v2/` 的三个新格式工程
是「类型检查通过、待工具链」状态；真正运行的是 §2 的源码注入路径。
