# Horizon Rust 模组移植：进度与交付说明

最后更新：2026-11-18（六个模组已在真实游戏里跑通并截图验证）

## 结论速览

**六个** Horizon 原生 Rust 模组：三个 MIT Java 模组移植 + 三个原创实现，
**代码、单测、整包真实编译、真机运行四层验证全部完成**：

| 模组 | 来源 / 许可 | 单测 | 真机运行 | 状态 |
|---|---|---|---|---|
| `whenchat-horizon` | ErikEdits/WhenChat · MIT | 7/7 | ✅ HUD 显示 `[HH:mm:ss]` 前缀 | ✅ 完成 |
| `chattimestamps-horizon` | PlagueTR/ChatTimestamps · MIT | 19/19 | ✅ 灰色前缀 / hover / 设置页三模式 | ✅ 完成 |
| `timeformatter-horizon` | m3z0id/TimeFormatter · MIT | 16/16 | ✅ `<t:1704067200:t>` → `08:00` | ✅ 完成 |
| `ports/coords.rs` | 原创（对应 Simple Coordinates 类） | 7/7 | ✅ 左上角 `X -55.5 Y 73.0 Z 7.5` / `S (+Z, 0°)` / `Overworld` | ✅ 完成 |
| `ports/durability.rs` | 原创（对应 Durability Viewer 类） | 6/6 | ✅ 右上角头盔 `23%`、靴子 `10%`，随磨损变色 | ✅ 完成 |
| `ports/autoreconnect.rs` | 原创（对应 Auto Reconnect 类） | 10/10 | ✅ 设置页可点可存（断线重连需真实服务器，见下） | ✅ 完成 |

> **本机（Windows）编译与真机验证全过程见 [`WINDOWS-BUILD.md`](WINDOWS-BUILD.md)** —— 含
> `native-build` 为何不可用、等价的手工注入编译路径、可复现命令，以及两轮共 4 个真 bug。
> 测试清单见 [`TESTING.md`](TESTING.md)；交付清单与发布步骤见 [`DELIVERY.md`](DELIVERY.md)。

**真机验证的关键结论**（截图在 `build-win/`）：用启动器自带的 Rust 1.99.1 把六个模组注入
Horizon 9ee13cd0 源码编译出 `horizon-client.exe`。发一条
`see you at <t:1704067200:t> ok`，HUD 显示为
`[11:44:59] [11:44:59] <Player32510> see you at 08:00 ok` ——
WhenChat 前缀、Chat Timestamps 灰色前缀、Time Formatter 时间戳三者同时生效；
同屏左上角是坐标面板（`hud-coords.png`），右上角是装备耐久条（`hud-durability.png`）；
Options > Mods 里六行都在、四个带 Settings 按钮可点可落盘（`mods-page.png`、`*-settings.png`）。

> ⚠️ **新版格式（`horizon-mod.toml` + trait 模组）已写好**（`native-v2/`，目前四个工程），但
> `horizon-launcher native-build` 在本机不可用：启动器要求 `nightly-2026-11-13-9f42064`
> 的 mod 工具链，而官方从未发布过任何 `sdk-*` release。因此本机走的是「源码注入 + 本机 Rust」
> 的等价路径，产出的是带模组的完整游戏二进制。
>
> ⚠️ **trait 形态的 API 缺口**：`Host` 没有 `item max_damage`，也**没有连接服务器的方法**，
> 所以 `durability` 与 `autoreconnect` 只能在注入形态下完整工作；`coords` 两种形态都可以
> （`native-v2/coords` 已通过类型检查）。等官方补齐 API 后这两个可以平移。



**验证分两层，均已通过：**
1. **整包真实编译（最权威）**：在 Docker Linux 容器（`rust:1.95` + 真实 Horizon `9ee13cd0` 源码）里跑 `native.py check all`，把三个模组的 helper 真实注入游戏源码后编译整个 `horizon-client`，结果 `{"mods": [...], "check": "ok"}`。这一层抓出并修复了 stub 测试发现不了的 4 个真 bug（helper 格式、binding 名、Text 类型、dead_code）。
2. **离线单元测试**：用本地 stub 复刻**引擎侧** `Text`（`horizon_client::text::Text`，扁平 `Vec<Span>`）模型，`include!` 真实的注入源文件跑行为测试，共 42/42 通过。

> ⚠️ 早期版本曾把 Text 当成 mod 侧组件树（`mod_api::text::Text`），据此写的 stub 测试全绿但**是错的**；整包编译揭穿后已全部改为引擎 Text 模型重写。README 正文中的 Text/签名描述以本页「关键 API 事实（最终修正）」为准。

## 选型标准

1. **许可证干净**：原模组源码为 MIT/Apache 等宽松许可，允许开源移植版；贴图等素材另行确认。
2. **纯客户端**：不依赖方块实体、网络包、实体 AI 等当前 API 缺口。
3. **未被移植**：不在 playhorizon.dev/mods 现有目录中。
4. **范围小**：核心逻辑可在一两百行 Rust 内完成，适合作为第一个练手项目。

## 候选模组

| 优先级 | 原模组 | 许可证 | 原实现 | Horizon 对应钩子 | 结果 |
|---|---|---|---|---|---|
| 1 | [WhenChat](https://github.com/ErikEdits/WhenChat) | MIT © ErikEdits | 一个 mixin 拦 `ChatHud#addMessage`，给消息加 `[HH:mm:ss]` 前缀；无配置 | `client_chat_received` | 已移植 |
| 2 | [Plague's Chat Timestamps](https://github.com/PlagueTR/ChatTimestamps) | MIT | 可配置时间戳：12/24 小时、毫秒、日期、颜色、hover 显示；依赖 Cloth Config + ModMenu | 同上 + `native_settings` / `native_settings_click` / `native_mods` | 已移植（子集） |
| 3 | [Time Formatter](https://github.com/m3z0id/TimeFormatter) | MIT © M3z0id | 把聊天里的 Discord 风格 `<t:...>` 时间戳转成本地时间 | 同上 | 已移植 |

备选方向：`item_tooltip` 和 `add_attribute_tooltips` 钩子已确认存在，若要做第四个项目可换成物品提示类模组。

## 关键 API 事实（整包真实编译修正后的最终版本）

- Horizon 有两种模组：`.hmod` 内容包（纯数据，无代码）和 `.hnmod` 原生模组（Rust 代码，启动器在本地编译进游戏）。
- 原生模组有两条写法：mixin helper（本项目采用，`native.py` 支持打包）和 typed API / `impl Hooks`（较新的稳定路线）。mixin helper 把 helper 注入游戏内部函数，能力强但依赖内部签名。
- **注入目标真实签名**（`native_hooks.rs`，已由整包编译确认）：
  `Game::native_client_chat_received(&mut self, system: bool, overlay: bool, sender: u128, chat_type: Option<&str>, message: &mut Text, cancel: &mut bool)`。
  是**展开的 6 个参数**，没有 `ClientChatReceivedCall` 这种 carrier。toml 的 bindings 按真实参数名绑定（`&mut *self` / `system` / `overlay` / `&mut *message`）。`overlay=true` 是动作栏（跳过）；替换 `message` 即改变显示文本。
  - 早期误以为参数是 `mod_api::__private::ClientChatReceivedCall`（旧 commit `85b5bcca` 的 API），整包 check 报 `E_INVALID_BINDING: binding 'call' must be a target parameter`。已全部改成展开参数。
- **Text 类型是引擎侧的 `horizon_client::text::Text`**（mixin helper 注入到 `native_hooks.rs` 后拿到的真实类型），不是 `mod_api::text::Text`。它是扁平结构：
  `Text { pub spans: Vec<Span> }`，`Span { text: String, style: Style }`，`Style { color: Option<u32>, hover: Option<Box<Text>>, ... }`。
  常用 API：`.plain()`、`.literal(s)`、`.colored(s, u32)`、`.styled(s, Style)`、`.push()/.append()`、`.spans`；颜色名经 `horizon_client::text::named_color(name) -> Option<u32>` 转换。**没有** `.children` / `.content` / `TextContent` / `TextHover` / `§` 自动解析。
  - ⚠️ 早期整套代码基于错误的 `mod_api::text::Text` 组件树模型（能跑通自造 stub 但整包必错），已全部按引擎 Text 重写：whenchat 用 `plain()+Text::literal`；chattimestamps 用 `Text::colored+push` 拼前缀、`span.style.hover=Box<Text>`；timeformatter 遍历 `spans` 改 `span.text`（天然保留样式）。
- **horizon-mixins helper 格式**：每个 helper 文件必须**恰好一个私有顶层 `fn`**，无顶层 `use`/attrs/泛型/async；所有 `use` 和辅助函数移进函数体（嵌套 `fn`）。`pub(crate)` 会触发 `E_INVALID_HELPER`。`runtime.rs` 是 `[[module]]` 普通模块，不受此限制。
- **pinned 版本**：三个模组均 pin Horizon commit `9ee13cd038559db05de6a4ced4496028ce409564`（hooks API = 645），含全部所需 flat 注入点，与官方 mod（smp，同样 hooks 645）一致。曾误 pin `85b5bcca...`（mod-api-gen 重构版，无 flat 注入点，`E_TARGET_NOT_FOUND`），已修正。
- `-D warnings` 下注入代码里未使用的函数/import 会编译失败：给仅供测试用的纯函数加 `#[allow(dead_code)]`，去掉未用的 `use`。
- 游戏 crate `horizon-client` 已依赖 `chrono`（features: clock, std），模组自身也声明同一依赖以便独立 `cargo test`。

## 项目一：whenchat-horizon

目录：`whenchat-horizon/`，注入目标 `Game::native_client_chat_received`。

- `runtime.rs`：本地时区前缀 `local_prefix()`（`chrono::Local::now().format("%H:%M:%S")`），含 `format_utc` 等纯函数与单元测试（纯函数加 `#[allow(dead_code)]`，因游戏只用本地时区前缀）。
- `chat.rs`（单私有 fn）：跳过动作栏 → `message.plain()` 读原消息 → 生成 `[HH:mm:ss]` 前缀 → `*message = Text::literal(decorated)` 写回；不设置 `cancel`。绑定展开参数 `(game, system, overlay, message)`。
- 配置清单：`horizon-mixin.toml`（2 个注入目标）、`compatibility.json`（pin 钩子所在的 main commit）、LICENSE、NOTICE.md 齐备。
- 验证：离线 7/7（4 个 runtime + 3 个聊天行为）；整包 `native.py check` 通过。

## 项目二：chattimestamps-horizon

目录：`chattimestamps-horizon/`，4 个注入目标：`native_client_chat_received`（时间戳）、`native_mods`（模组列表行）、`native_settings`（3 个设置按钮）、`native_settings_click`（持久化点击）。

- `runtime.rs`：`Settings`（`enable_mod` / `format` / `enable_hover`）、Java SimpleDateFormat → chrono pattern 转换 `to_chrono_pattern`、样式分段 `split_segments`、`timestamp_now`；12 个单元测试覆盖格式字母、带引号字面量、毫秒、颜色码映射、设置缺省值/读取、按钮循环等。
- `chat.rs`（单私有 fn）：前缀模式用 `Text::colored` + `push` 把时间戳 spans 拼在原消息前、保留原 span 样式；hover 模式给每个 span 的 `style.hover = Some(Box<Text>)` 挂时间戳，可见文本不变；动作栏与禁用态原样跳过。绑定 `(game, overlay, message)`。
- `settings.rs` / `click.rs` / `list.rs`：设置页与点击持久化，模组列表登记。三函数均为单私有 fn（满足 mixins 校验）。
- 配置清单：`horizon-mixin.toml`、`compatibility.json`、LICENSE、NOTICE.md 齐备。
- 验证：离线 19/19（12 个 runtime + 7 个行为）；整包 `native.py check` 通过。

## 项目三：timeformatter-horizon

目录：`timeformatter-horizon/`，2 个注入目标：`native_client_chat_received`（聊天替换）、`native_mods`（模组列表行）。

- `runtime.rs`：从 m3z0id/TimeFormatter 的 `Formatter.java` + `ChatListener.java` 逐行移植——手写解析 `<t:SECONDS[:STYLE]>`（数字 1-13 位或 `now`/`yesterday`/`tomorrow`，默认样式 `f`，上限 `8640000000000`），7 种样式 `f/F/d/D/t/T/R`（相对时间按原算法），`<<t:...>>` 双尖括号字面量原样保留。无 `regex` 依赖；格式化函数注入时区参数，10 个单元测试用固定 UTC 校验 7 种格式、特殊值、相对时间与解析边界。
- `chat.rs`（单私有 fn）：遍历引擎 Text 的 `spans`，对每个 `span.text` 就地替换（天然保留该 span 的颜色/hover 样式）；跳过动作栏。绑定 `(overlay, message)`。
- `list.rs`：模组列表登记。
- 配置清单：`horizon-mixin.toml`、`compatibility.json`、LICENSE（保留 M3z0id 2025 署名）、NOTICE.md 齐备。
- 验证：离线 16/16（10 个 runtime + 6 个行为）；整包 `native.py check` 通过。
- 未移植：原模的图形化"存时间戳"界面（TimeUIScreen / 日期时间输入控件）与 Fabric API 依赖——属于附加功能，核心的聊天自动格式化已完成。

## 复现验证命令（Docker Linux 容器，已验证）

本机 Windows 无可用 cargo，验证统一在 Docker 容器 `hn-build`（`rust:1.90-slim` 基础 +  rustup 装的 `1.95.0` + 全部系统开发库）里完成。容器 Config.Env 写死了失效代理 `60145`，故每条 cargo 命令都要 `env -u` 清掉代理，走 rsproxy 直连。

```bash
# 1) 整包真实编译（最权威）：需已在 native/target/horizon/<rev>/ 备好源码缓存
docker exec hn-build bash -c 'cd /workspace && \
  env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u all_proxy \
  python3 native/tools/native.py check all'
# 预期：{"mods": ["chattimestamps","timeformatter","whenchat"], "horizon": "9ee13cd...", "check": "ok"}

# 2) 离线单测（引擎 Text 模型）：把每个 <mod>-horizon/ 复制进容器后 cargo test --offline
docker cp <mod>-horizon hn-build:/tmp/<mod>
docker exec hn-build bash -c 'cd /tmp/<mod>/verify && \
  env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u all_proxy \
  cargo test --offline'
# 预期：whenchat 7/7、chattimestamps 19/19、timeformatter 16/16 全绿
```

容器环境搭建细节（`cargo config.toml` 的 rsproxy 镜像、华为云 rustc/apt 源、系统库清单）见 [`DOCKER-BUILD.md`](DOCKER-BUILD.md) 与工作日志。

## 本机游戏环境（2026-11-17 只读探测）

这台电脑上**有完整的 Java 版 Minecraft**（未改动任何文件）：

- 位置：`E:\Minecraft\`，PCL2 启动器 `E:\Minecraft\Plain Craft Launcher 2.exe`。
- `.minecraft\versions` 下 4 个版本，jar 齐全：`1.20.1`、`1.20.1-Fabric 0.16.13`、`1.20.2-Forge 48.1.0`、`1.21.4-Forge 54.0.34`。
- `assets`（3 套索引）、`libraries`（12 项）存在；`.minecraft` 根目录已有若干 Java 模组 jar（JEI 1.21.4、苹果皮等）。
- Java 运行时：`C:\Program Files\Java\jdk-23`、`jre1.8.0_491`、`javafx-sdk-21`，Eclipse Adoptium `jdk-17`。
- 另有 `D:\Prism`（Prism 启动器目录）、`G:\Games`、`E:\EpicGames`。
- `C:\Users\ASUS\AppData\Roaming\.minecraft` 只有一个 `cache` 目录，是残留缓存，不是游戏。

意义：实体游戏资源齐备，将来 Horizon 需要"从正版 Java 准备资源"时，素材来源不缺；但注意 Horizon 目标版本是 26.3，与本机现有的 1.20.1/1.21.4 不同，届时可能仍需针对 26.3 准备一次资源。

## 关于微软账号（2026-11-16 核实官方仓库原文）

- **官方启动器**强制要求：登录微软账号且该账号拥有 Minecraft Java Edition（"owner gate"，2026-11-09 起离线名字被移除，写入 GOVERNANCE 的不可投票规则）。但**首次在线验证过后，断网也能玩**（记住上次答案，无 session 离线启动）；单机 / LAN / 离线模式服务器不要求在线验证。
- 网上 11 月初"Horizon 还不能登录微软账号"的报道**已过时**：Mojang 已批准其 app registration（见 GOVERNANCE.md），登录现已可用。
- **对模组测试的意义**：走**启动器 Play 必须登录**拥有 Java 版的微软账号（owner gate）；首次验证后可断网/离线名字玩。只有直接跑 `native.py build` 出的游戏二进制 + `--name <名字>` 才能完全不登录。游戏资源仍需从你自己那份正版 Java 26.3 准备一次（Horizon 不含任何 MC 文件）。

## 下一步（真正上游戏前）

详见 [`TESTING.md`](TESTING.md)，摘要：

1. ~~整包编译确认~~ **已完成**：容器内 `native.py check all` 三模组全绿（真实 Horizon 9ee13cd0 + rustc 1.95）。
2. 本机跑 `horizon-prepare --prepare` + `sounds` + `--check` 准备 26.3 资源（资源已 100% 下完，JDK 25 已备好）。
3. 用 `horizon-launcher native-build ... --trust-native-code` 构建，`--name` 离线启动，按 TESTING.md 的清单做真机实测（这是唯一剩下的环节）。
4. 发布前确认：原模组 MIT 保留署名；移植版同样用 MIT，并在 NOTICE 中注明原作者。

## 许可证说明

三个原模组均为 MIT，允许修改和再发布，条件是保留版权与许可声明。因此这三个 Rust 移植版可以在保留署名与 NOTICE 的前提下开源。Horizon 主仓库本身是 LGPL-3.0-or-later（早期代码 MIT）；单独发布模组源码没有冲突，但分发打包好的游戏二进制时要注意 LGPL 义务。
