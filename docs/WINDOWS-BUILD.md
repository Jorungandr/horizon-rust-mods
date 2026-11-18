# 本机（Windows）编译与真机验证

最后更新：2026-11-18

本文记录**六个模组**（三个 Java 移植 + 三个原创）在**这台 Windows 机器**上真正编译进游戏、
并跑起来验证的全过程，包括踩到的坑和最终结论。复现只需照抄命令。

---

## 1. 为什么不能直接 `native-build`

启动器 0.2.70 只认新格式（`horizon-mod.toml`），旧包的 `horizon-mixin.toml` 会被拒：
`error: unsupported package path horizon-mixin.toml`。

改成新格式后，探针工程（`native-v2/probe/`）被正确识别，但卡在下一层：

```
The mod tools for nightly-2026-11-13-9f42064 aren't ready yet
```

`native-build` 需要用「该实例精确源码版本」的 mod 工具链（SDK）来编译，
这份工具链由启动器按需下载。查证结果：

- GitHub 上 `horizon-rust/horizon` 的全部 200 个 release 里**没有任何 `sdk-*` release/tag**；
- 启动器内嵌的 mixins pin 是 `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`，与本机 9ee13cd0 源码树里的
  `horizon_mod_package::mixins_tool::REV` 一致 —— 说明 pin 填对了，缺的是工具链本体。

**结论：官方 `native-build` 路径当前不可用，且不是我方代码问题。**

## 2. 等价路径：源码级注入 + 本机 Rust

`native-build` 的本质是「把模组代码注入实例源码后编译出一个带模组的游戏版本」。
这件事可以手工做，效果等价：

1. 用本机已有的 Horizon `9ee13cd0` 源码树（`horizon-mods-staging/native/target/horizon/9ee13cd0…`）；
2. 把三个模组作为私有模块注入 `crates/horizon-client/src/game/native_hooks/`，
   并在游戏的钩子函数里调用；
3. 用启动器自带的 Rust 1.99.1（`horizon-launcher rust install` 安装）编译。

### 关键环境

| 项 | 值 |
|---|---|
| Rust | `%APPDATA%\horizon-launcher\rust\1.99.1-x86_64-pc-windows-gnu`（`x86_64-pc-windows-gnu`） |
| 链接器 | `D:\MinGW\bin\gcc.exe`（MinGW-W64 15.2.0） |
| 依赖镜像 | rsproxy sparse（`build-win/cargo-home/config.toml`），`cargo fetch --locked` 约 865 MB |
| 编译 profile | `--profile fast`（继承 release，`lto = false`、`codegen-units = 16`、`incremental = true`） |
| 产物 | `build-win/horizon-src/target/fast/horizon-client.exe`（约 147 MB） |

### 命令

```bash
# 1) 注入三个模组（重拷源码树 + 打补丁）
python tools/inject_all_ports.py

# 2) 编译客户端
bash tools/build_win.sh          # -> target/fast/horizon-client.exe

# 3) 编译同源码版本的 horizon-ctl（避免控制协议代差）
bash tools/build_ctl.sh          # -> target/fast/horizon-ctl.exe

# 4) 准备一个独立游戏目录（复制官方 nightly 已准备好的 26.3 资源，不动原安装）
python tools/make_game_port.py

# 5) 无头验证
bash tools/verify_ports.sh       # 聊天 + 三种设置模式 + Options>Mods 截图
bash tools/verify_settings.sh    # 设置页读写往返
```

## 3. 注入点

`crates/horizon-client/src/game/native_hooks.rs`：

| 位置 | 注入内容 |
|---|---|
| 模块声明 | `#[path = "native_hooks/ports_*.rs"] pub(super) mod ports_*;` 六个 |
| `native_client_chat_received` | `whenchat` → `chattimestamps` → `timeformatter` 依次改写 `message` |
| `native_mods` | 六个 `NativeMod` 行（Options > Mods 列表） |
| `native_settings` / `native_settings_click` | 四个模组的设置页与点击（chat timestamps、coords、durability、autoreconnect） |
| `native_hud` | `coords::hud` 与 `durability::hud`，画进同一个 HUD 批次 |
| `native_tick` | `autoreconnect::tick`，推进重连倒计时 |
| `native_client_level_load` | `autoreconnect::joined`，进入世界即重置重试预算 |

`pub(super)` 是因为 `game/remote.rs` 也要调用重连模块。

`crates/horizon-client/src/game/remote.rs`：

| 位置 | 注入内容 |
|---|---|
| `connect_server_with_options` 开头 | `ports_autoreconnect::record(地址, 名字, cookies, lan, realm)` |
| `leave_server` 的 `finish_perf()` 之后 | `ports_autoreconnect::on_disconnect(self, reason.is_some())` |
| `chat_received` | 删掉空注册表守卫（见下） |

另有**一处必要的引擎改动**（`game/remote.rs` 的 `chat_received`）：

```rust
if horizon_client::mod_api::mods().is_empty() {
    return true;                       // ← 删掉
}
```

原逻辑是「没有注册进 `mod_api` 的 mod 就跳过钩子」。我们的模组是编译进二进制的，
不经过 `mod_api` 注册，这个守卫会让所有聊天行静默跳过。删掉后空列表 dispatch 仍是空操作。

## 4. 真机验证结果

环境：RTX 4060 Laptop / Vulkan / `horizon-ctl launch --test-terrain --render-distance 4 --wait`。

### 4.1 三条链路同时生效

发送 `hello ports` 与 `see you at <t:1704067200:t> ok` 后，`horizon-ctl messages`：

```json
{
  "chat": [
    "[11:44:59] [11:44:59] <Player32510> see you at 08:00 ok",
    "[11:44:59] [11:44:59] <Player32510> hello ports"
  ],
  "styled": [
    {"extra": [
      {"color": "gray", "text": "[11:44:59] "},
      {"text": "[11:44:59] <Player32510> see you at 08:00 ok"}
    ], "text": ""}
  ]
}
```

- 第一个灰色 `[11:44:59] ` —— **Plague's Chat Timestamps** 的前缀（`&7` → gray）
- 文本里的第二个 `[11:44:59] ` —— **WhenChat** 的前缀
- `<t:1704067200:t>` → `08:00` —— **Time Formatter**

截图 `build-win/ports-proof.png` 与游戏 HUD 完全一致。

### 4.2 hover 模式

把 `enable_hover` 置为 `true` 后再发一行：

```json
{"extra":[{"hover_event":{"action":"show_text","value":{"extra":[{"color":"gray","text":"[11:45:04] "}],"text":""}},
           "text":"[11:45:04] <Player32510> hover mode line"}],"text":""}
```

前缀消失、时间戳变成 tooltip —— 与原模组的 hover 行为一致。

### 4.3 关闭模组

`enable_mod = false` 后新行只剩 WhenChat 前缀，没有任何灰色时间戳。

### 4.4 Options > Mods 设置页

`build-win/mods-page.png`：

```
Mods
  [Browse Mods...]
  [WhenChat 0.1.0: ON]
  [Plague Chat Timestamps 0.1.0:] [Settings]
  [Time Formatter 0.1.0: ON]
  [Open Mod Folder]  [Done]
```

点 `Settings` 进入本模组页面（`build-win/chattimestamps-settings.png`），三个按钮：

```
[Timestamps: ON]
[Format: Gray [HH:mm:ss]]
[On hover only: OFF]
```

点击 `On hover only: OFF` 后：

- 按钮变 `On hover only: ON`
- 落盘文件 `horizon-mods/.horizon-mod-data/chattimestamps/`：
  `enable_hover = true`、`enable_mod = true`、`format = &7[HH:mm:ss] `（含尾随空格）

## 5. 真机验证才暴露出来的六个问题

### 5.1 `Text` 有两份表示，只改 spans 会让组件 JSON 过期

`horizon_client::text::Text` 同时持有 `spans`（游戏绘制用）和私有的 `source`
（`component()` 返回的组件 JSON）。原地改 `span.text` 后，`spans` 是新的、
`source` 还是旧的 —— `horizon-ctl messages` 的 `styled` 字段就会显示原始 `<t:...>`。

修法：用 `Text::styled(text, span.style.clone())` 逐 span 重建整个 `Text`，
两份表示同步更新。`whenchat` 本来就用 `Text::literal` 整体替换，不受影响。

### 5.2 `Game::native_setting` 会 trim 所有值，而时间戳格式的尾随空格有语义

默认格式是 `"&7[HH:mm:ss] "`（结尾一个空格）。点击任意设置项会把三个值写盘，
再读回来时 `trim()` 掉了尾随空格 → 与预设 `"&7[HH:mm:ss] "` 不等 →
设置页显示 `Format: Custom`，且时间戳会和正文粘在一起。

修法：读设置后，若某个预设 `trim()` 后与读到的值相同，就还原成预设原文
（`ports_chattimestamps::read_settings`）。

### 5.3 `chat_received` 的空注册表守卫

见第 3 节。这是三个模组当初在 Docker 整包编译里没能发现的：那里只验证「能编译」，
没有真正发一条聊天。

### 5.4 `native_camera()` 在 HUD 阶段是 `None`

坐标面板第一版用 `native_camera()`，进游戏后**一条都没显示**。原因：`native_camera()`
读的是 `camera_frame`，它只在世界渲染那一帧被填；HUD 提取阶段的时机不保证有值。
改用 `native_player()`（进入世界后 `self.player` 一直在）立刻正常。

> 这也是给后续模组的经验：**HUD 里要玩家数据就用 `native_player()`**，
> 要相机矩阵才用 `native_camera()`，并且必须有 `None` 分支。

### 5.5 成就弹窗挡住了右上角面板，被误判成「没画」

耐久面板默认在右上角。第一轮验证时截图里看不到，以为绘制失败；实际是同一帧弹了三个
成就 toast（`Cover Me With Diamonds`、`Suit Up`…），整块盖住了。第二轮没有新成就，面板就出现了。

> 验证 HUD 时要么换一个角落，要么用已经解锁过的世界——**别在弹成就的那一帧判定失败**。

### 5.6 设置页注入漏了一个模组，按钮就静默消失

`SETTINGS_PORTS` 里只写了 chat timestamps / coords / durability，漏了 autoreconnect。
结果 Mods 列表里 Auto Reconnect 没有 Settings 按钮——**不是 UI 问题，是注入清单漏项**。
Options > Mods 的 Settings 按钮由 `native_settings` 返回的按钮驱动，返回空就不显示。

---

## 6. 迁移到新格式（`horizon-mod.toml`）



`native-v2/<id>/` 下是三个模组的**新版（trait 模组）工程**：

```
native-v2/whenchat/
  horizon-mod.toml     [mod] id/version、[compatibility] horizon_revision、[api] path
  api/Cargo.toml       package mod-<id>，依赖 horizon-client
  src/lib.rs           实现 mod_api::Hooks（mods / settings / settings_click / client_chat_received）
  src/runtime.rs       与原 mixin 版逐字节相同的纯逻辑
```

与旧 `.hnmod` 的对应关系：

| 旧（`horizon-mixin.toml`） | 新（`horizon-mod.toml` + `Hooks`） |
|---|---|
| `[[module]]` + `runtime.rs` | `src/runtime.rs`（普通 `mod`） |
| `[[injection]] chat`（helper 改 `&mut Text`） | `fn client_chat_received(&self, host, event) -> Flow` |
| `[[injection]] list`（helper push `NativeMod`） | `fn mods(&self, host, event)` |
| `[[injection]] settings` | `fn settings(&self, host, event)` |
| `[[injection]] click` | `fn settings_click(&self, host, event)` |
| `game.native_setting(...)` | `host.setting(id, key)` / `host.set_setting(...)` |

`native-v2/Cargo.toml` 是一个只用于**类型检查**的工作区（`exclude = ["probe"]`，
并复制了 Horizon 的 `[patch.crates-io]`）。这不是启动器需要的文件，只是让这三个工程
能对着本机 Horizon 源码 `cargo check`。

**三个工程已通过类型检查**（对着真实 9ee13cd0 源码的 `mod_api`）：

```bash
cd native-v2
export CARGO_HOME=../build-win/cargo-home
export PATH="$APPDATA/horizon-launcher/rust/1.99.1-x86_64-pc-windows-gnu/bin:/d/MinGW/bin:$PATH"
export CARGO_TARGET_DIR=../build-win/check-target
cargo check --offline
#   Checking mod-timeformatter v0.1.0
#   Checking mod-whenchat v0.1.0
#   Checking mod-chattimestamps v0.1.0
#   Finished `dev` profile ... in 3.15s
```

`Cargo.lock` 是从 `build-win/horizon-src/Cargo.lock` 复制的（离线解析要用它的精确版本，
例如 `clipboard-win 5.4.1` 已被上游 yank，重新解析会失败）。

> `native-build` 仍需官方 mod 工具链；本机不可用。所以 trait 模组工程目前是
> **「API 正确、类型检查通过、工具链可用即可 `native-build`」** 的状态；
> 真正跑起来的仍是第 2 节那条源码注入路径。

## 7. 已知遗留

- **静态注入只能整包重编**：改一行模组代码要重编 `horizon-client`（增量约 2–5 分钟）。
  `native-build` 可用后，trait 模组可以按实例单独构建，不必动游戏源码。
- **`--game` 指向的目录必须带已准备好的资源**：`horizon-ctl` 会读
  `research-lock.json` → `.local/horizon-client/26.3/horizon-resources.json`。
  `tools/make_game_port.py` 就是为此复制一份官方 nightly 的游戏目录。
- **`verify_*.sh` 每次必须先复制 exe**：漏了这一步会拿旧二进制跑，症状是「代码改了没效果」。
  第一次排查这个 bug 就多花了两轮编译。
- 安全策略会拦 `reg.exe`，脚本里 `quit` 之后可能出现一次拦截记录，不影响验证结果。
