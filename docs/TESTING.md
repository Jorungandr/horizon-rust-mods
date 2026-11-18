# Horizon 原生模组测试指南 & 下一步

最后更新：2026-11-17

本文回答三个问题：**游戏文件在哪 / 26.3 准备得怎样**、**我自己要测什么**、**下一步干什么**。

---

## 一、游戏文件位置与 26.3 准备状态

### 你现有的 Java 版 Minecraft（只读探测，未改动）

- **根目录**：`E:\Minecraft\`
- **启动器**：PCL2（`Plain Craft Launcher 2.exe`），另有 `D:\Prism`
- **已有版本**：`1.20.1`、`1.20.1-Fabric 0.16.13`、`1.20.2-Forge 48.1.0`、`1.21.4-Forge 54.0.34`
- **资源**：`assets`（多套索引）、`libraries`、已有若干模组 jar
- **Java**：原本只有 17 / 21 / 23

### 本次新增的 26.3 文件（全部落在同一个 `.minecraft`，不碰旧版本）

| 项目 | 路径 | 状态 |
|---|---|---|
| 版本 JSON | `E:\Minecraft\.minecraft\versions\26.3\26.3.json` | ✅ SHA1 校验通过 |
| 客户端 jar | `E:\Minecraft\.minecraft\versions\26.3\26.3.jar`（39.6 MB） | ✅ SHA1 校验通过 |
| 服务端 jar | `E:\Minecraft\.minecraft\server-26.3\server.jar`（59.4 MB） | ✅ SHA1 校验通过（Horizon 资源准备必需） |
| 库文件 | `E:\Minecraft\.minecraft\libraries\`（114 个） | ✅ 114/114 校验通过 |
| 资源索引 | `E:\Minecraft\.minecraft\assets\indexes\34.json` | ✅ 就位 |
| 资源对象 | `E:\Minecraft\.minecraft\assets\objects\`（约 487 MB / 5149 个） | ✅ 5149/5149 = 100%，逐个 SHA1 校验通过 |
| JDK 25 | `E:\Minecraft\jdk-25\jdk-25.0.5.0.2+1\bin\java.exe` | ✅ `java -version` = 25.0.5 LTS（微软 OpenJDK） |

> 说明：Horizon **不含任何 Minecraft 文件**。它从"你自己那份正版 Java 26.3 + 服务端 jar"里提取资源。这就是为什么必须准备 26.3（你原有的 1.20.1/1.21.4 版本不适用）。
>
> 下载备注：Mojang 官方 CDN 中途 502，资源对象改用 BMCLAPI 镜像（`/assets/<前2位>/<完整hash>`）逐个 SHA1 校验补齐；JDK 改用微软 OpenJDK（GitHub CDN 被代理拦）。下载脚本均在 `tools/`。**资源准备（3.1）所需文件现已全部就绪。**

---

## 二、测试分三层（从易到难）

### 第 1 层：离线单测（引擎 Text 模型）—— 已完成 ✅

**不需要游戏、不需要账号、不需要网络**，在 Docker 容器里跑（本机无 cargo）：

| 模组 | 离线单测（runtime + 聊天行为） |
|---|---|
| WhenChat | 7/7 |
| Chattimestamps | 19/19 |
| TimeFormatter | 16/16 |

复现命令见 `README.md`。测试用 stub 复刻**引擎侧** `Text`（`horizon_client::text::Text`，扁平 spans 模型），`include!` 真实注入源文件编译运行。早期版本用的组件树模型是错的，已全部重写。

### 第 2 层：整包真实编译 `native.py check` —— 已完成 ✅

这一层把模组源码打到**真实 Horizon 源码**上编译整个游戏，能发现 stub 发现不了的问题。已在 Docker 容器（`rust:1.95` + 真实 Horizon `9ee13cd0`）跑通：

```
{"mods": ["chattimestamps", "timeformatter", "whenchat"], "horizon": "9ee13cd...", "check": "ok"}
```

这一层实际抓出并修复了 5 类 stub 发现不了的真 bug（helper 格式 / binding 名 / Text 类型 / dead_code / rustc 版本），是目前最权威的验证。环境与命令细节见 `README.md`、`DOCKER-BUILD.md`。通过后已用 `native.py package all` 重打三个 `.hnmod`。

### 第 3 层：真机游戏内实测 —— 未做（待环境 + 资源准备）

这是最终验收。需要先准备游戏资源，再用启动器构建带原生模组的游戏进测。

#### 步骤 3.1：准备 26.3 资源（Windows 本机就能做，JDK 25 已就位）

在一个 Horizon checkout 里（或下载官方 release 里的 `horizon-prepare`）：
```powershell
$env:JAVA_HOME = "E:\Minecraft\jdk-25\jdk-25.0.5.0.2+1"
.\horizon-prepare.exe --prepare `
    --minecraft-dir "E:\Minecraft\.minecraft" `
    --server-jar "E:\Minecraft\.minecraft\server-26.3\server.jar"
.\horizon-prepare.exe sounds --minecraft-dir "E:\Minecraft\.minecraft"
.\horizon-prepare.exe --check    # 应返回 0，表示资源完整
```
> 资源会落到 checkout 的 `.local\horizon-client\26.3\`（**私有，切勿提交**）。sounds 约 380 MB，从你在下的是完整 asset store 复制，每个文件按索引 SHA-1 校验。等后台 assets 下载完再做这步。

#### 步骤 3.2：用启动器构建带原生模组的游戏

原生模组没上架 registry 时，玩家不能从 Mods 页面装 `.hnmod`；自己测要用启动器的 `native-build` 从**源码文件夹**构建（`mixins/<id>/`，不是 `.hnmod` 文件），并显式信任代码。三个模组 pin 同一 commit `9ee13cd0`，可一次全传：
```
horizon-launcher native-build "G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/whenchat-horizon/mixins/whenchat" "G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/chattimestamps-horizon/mixins/chattimestamps" "G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/timeformatter-horizon/mixins/timeformatter" --trust-native-code
```
开发期可先跑 `horizon-launcher settings native-profile dev` 用非优化编译（改动后重建快约 6 倍）。首次 build 启动器会下载自己的 Rust，约 20–30 分钟。

登录：**启动器 Play 必须登录**拥有 Java 版的微软账号（2026-11-09 起的 owner gate，见官方 install 文档）；首次在线验证后可断网或用离线名字玩。只有直接用 `native.py build` 出的二进制 + `--name <名字>` 才能完全不登录。

✅ **版本兼容已预检（2026-11-17，两轮）**：

- **Nightly（启动器实际装的）**：最新 nightly-source tag `2026-11-13-9f42064`（commit `9f42064c`）→ `game/native_hooks.rs` 路径仍在，我们用到的 4 个钩子 `native_mods`、`native_settings`、`native_settings_click`、`native_client_chat_received` **签名 100% 一致，代码无需改动**。启动器装的就是 Nightly，**这条路是安全的**。
- **main 分支（仅供开发，启动器不装）**：main HEAD `3a6bea51`（11-17）已把 `game/native_hooks.rs` 重构成 `game/modding/` 下 400+ 个拆分文件 + typed API（`src/native_client_hooks.rs`），旧钩子函数被移走。官方明确说 **mixin helper（我们用的方式）"works on every Horizon release"**，且 horizon-mixins ≥0.6.0 支持用 `hook = "名字"` 代替 `target = 函数路径`，钩子函数移动也不会坏。将来若启动器的 Nightly 跟进这次重构，把 `horizon-mixin.toml` 的 `target` 改成 `hook`/`hook_version` 即可，我这边可随时适配。

⚠️ **别用 `--horizon latest` 来 build 本模组**：最新 tag 指向的是 11-13 凌晨 02:39 的 commit（`NATIVE_HOOKS_API=415`），比我们 pin 的 `9ee13cd0`（11-13 17:56，hooks 645）还早约 15 小时——`latest` 在这里反而更旧。用启动器 `native-build` 直接按模组 `compatibility.json` 的 pin（`9ee13cd0`）构建即可，不要加 `--horizon latest`。

若启动器装的 Nightly 版本比 pin 更旧导致 build 报 hook/绑定找不到：在启动器 Versions 页装回 `9ee13cd0` 对应的 Nightly（每个 Nightly 的 commit 都保留），或告诉我，我用 `native.py check <mod> --horizon latest` 复测并适配。

#### 步骤 3.3：进游戏逐项实测（测试点清单）

**WhenChat**（聊天时间戳前缀）：
- [ ] 普通聊天消息前出现本地时间 `[HH:mm:ss]`
- [ ] 动作栏（overlay）消息**不加**前缀
- [ ] 系统消息、玩家私聊表现符合预期
- [ ] 时区正确（与系统本地时间一致）

**Chattimestamps**（可配置时间戳，含 hover）：
- [ ] 前缀模式：格式串（如 `[HH:mm]`）正确渲染
- [ ] hover 模式：鼠标悬停显示精确时间
- [ ] 颜色码（`§`）正确显示为颜色，而非字面字符
- [ ] 设置页：开关 `enableMod`、改 `timestampFormat`、切 `enableHover` 后即时生效
- [ ] 动作栏消息跳过
- [ ] 模组列表里三项设置正确登记

**TimeFormatter**（Discord 风格时间戳）：
- [ ] `<t:秒:f>` 等 7 种样式（f/F/d/D/t/T/R）渲染正确
- [ ] `<t:秒>` 无样式时默认按 `f`
- [ ] `now` / `yesterday` / `tomorrow` 特殊值正确
- [ ] `<<t:秒:f>>` 双尖括号**原样保留**（字面量场景，不二次替换）
- [ ] 带逐字符颜色 / hover 的复杂消息，替换后样式不丢失
- [ ] `R` 相对时间（如 "2 days ago"）随时间正确变化

**通用检查**：
- [ ] 游戏能正常启动、进世界、无崩溃
- [ ] 聊天无模组报错（日志里无 panic / hook 解析失败）
- [ ] 三个模组同时启用不冲突

---

## 三、下一步行动清单（按当前真实状态）

> 状态更新（2026-11-17 深夜）：**整包真实编译 `native.py check all` 已全绿**。三个模组按引擎 Text 真实 API 重写后注入 Horizon `9ee13cd0` 编译通过；离线单测也同步改为引擎 Text 模型（42/42）；三个 `.hnmod` 已用修正后源码重打。只剩真机游戏内实测。

1. **✅ 已完成**：26.3 资源（5149/5149）、JDK 25、Docker 容器（rustc 1.95 + 全部系统开发库 + `horizon-mixins-cli`）。
2. **✅ 已完成**：修正 Horizon pin（`85b5bcca` → `9ee13cd0`，hooks 645）。
3. **✅ 已完成**：整包 `native.py check all` 三模组全绿。此过程抓出并修复了 5 类 stub 发现不了的真 bug：helper 格式、binding 名、Text 类型（组件树 → 引擎 spans）、dead_code/unused import、rustc 1.95 门槛。
4. **✅ 已完成**：离线单测重写为引擎 Text 模型（whenchat 7/7、chattimestamps 19/19、timeformatter 16/16）；三个 `.hnmod` 用修正后源码重打并解包校验。

### 3.1 剩下的唯一环节：真机游戏内实测

整包编译通过不等于游戏里行为正确。按第 3.3 节清单做最终验收：

- 资源准备（Windows 本机即可）：跑 `horizon-prepare --prepare` + `sounds` + `--check`（JDK 25 已就位）。
- 启动器 `native-build --trust-native-code` 构建带模组的游戏，用 `--name` 离线启动。
- 安装 `dist/hnmod/` 下三个 `.hnmod`，按清单逐项验证时间戳前缀、hover、设置页、Discord 时间戳等实际显示效果。
- 发布：三个原作均 MIT，保留 LICENSE + NOTICE；如需上架按 `horizon-mods` 提交流程审核。

> 说明：到此**代码层验证全部完成**（离线 42/42 + 整包真实编译全绿）。整包 check 的价值远超离线 stub——它揭穿了早期错误的 Text 模型并推动全部按引擎真实 API 重写。剩下的是真机层面的最终确认，需要在能运行 Horizon 游戏的环境里做。
