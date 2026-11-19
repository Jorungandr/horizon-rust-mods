# Horizon Ports — 九个 Rust 模组整合包

这是一个**已编译、带九个原生 Rust 模组**的 Horizon 客户端压缩包。模组不是外挂式 jar，而是直接注入并编译进 `horizon-client.exe`，所以启动方式跟普通 Horizon 不太一样——请用本包自带的 `START_GAME.cmd`，不要用启动器的 Play。

九个模组都已在真实游戏里跑通并截图验证（见 `evidence/`）。

---

## 一、三步开始玩

### 第 1 步：解压
把整个压缩包解压到一个**没有中文和空格**的路径，例如 `D:\HorizonPorts\`。路径里有中文可能导致启动脚本或游戏读资源出错。

### 第 2 步：准备游戏资源（只需一次）
Horizon 本体**不含**任何 Minecraft 的代码、贴图或声音，这些必须从你自己的正版 Minecraft Java Edition 导入：

1. 确认你已用官方启动器安装并运行过 **Minecraft Java 26.3**。
2. 准备好一个 **Java 25** 运行时（JDK 更稳妥）。
3. 双击 `PREPARE_RESOURCES.cmd`，按提示填三个路径：
   - `.minecraft` 目录（默认 `E:\Minecraft\.minecraft`）
   - 26.3 的 `server.jar`（默认 `E:\Minecraft\.minecraft\server-26.3\server.jar`）
   - Java 25 的家目录（默认 `E:\Minecraft\jdk-25\jdk-25.0.5.0.2+1`）

脚本会导入 26.3 资源、声音、字体、语言等，跑完打印 `[OK] Resources complete`。这一步要几分钟，并且会占用几个 GB 磁盘。已经导入过的文件会复用，失败了重跑即可。

### 第 3 步：启动
双击 `START_GAME.cmd`。它会自动用带模组的客户端、正确的资源和独立的存档/日志目录启动游戏，世界名默认是 `Ports Proof`（想换名字就编辑脚本里的 `--world` 那一行）。

进游戏后按 `Esc` → `Options` → `Mods`，能看到九行模组；其中七行带 `Settings` 按钮，点击可改设置并自动落盘。**列表可以滚动**——如果看不到最后两个（FPS HUD、Item Tooltip+），在列表上向下滚动即可。

> 操作键位：WASD 移动、鼠标视角、空格跳、F1 隐藏 HUD、E 背包、Esc 暂停。

---

## 二、九个模组分别做什么

| 模组 | 来源 | 效果 | 设置 |
|---|---|---|---|
| **WhenChat** | ErikEdits/WhenChat · MIT | 每条聊天前加本地时间 `[HH:mm:ss]` | 无 |
| **Chat Timestamps** | PlagueTR/ChatTimestamps · MIT | 可配置时间戳：12/24 小时、日期、颜色、hover 显示 | 有（3 项） |
| **Time Formatter** | m3z0id/TimeFormatter · MIT | 把聊天里的 `<t:1704067200:t>` 这类时间戳转成本地时间 | 无 |
| **Coordinates HUD** | 原创 | 左上角显示 X/Y/Z、朝向、维度 | 有（位置/小数位/开关） |
| **Durability HUD** | 原创 | 右上角显示装备耐久百分比，随磨损变色 | 有（显示开关等） |
| **Auto Reconnect** | 原创 | 非主动断线后按设定延迟自动重连，带重试上限 | 有（开关/延迟/次数） |
| **Chat Highlight** | 原创 | 把聊天里的关键词染成指定颜色 | 有（开关/关键词包/颜色/大小写） |
| **FPS HUD** | 原创 | 角落显示实时 FPS、最慢帧、1% low | 有（开关/位置/详细模式） |
| **Item Tooltip+** | 原创 | 物品提示里追加 ID、耐久、堆叠数 | 有（开关/ID/耐久/堆叠） |

真机截图里的聊天行是这样的：

```
[12:27:29] [12:27:29] <Player32510> see you at 08:00 ok
[12:27:29] [12:27:29] <Player32510> server look here important
```

第一个灰色时间戳来自 Chat Timestamps，第二个来自 WhenChat，`08:00` 是 Time Formatter 把 `<t:1704067200:t>` 转出来的；第二行里 `server`、`look here`、`important` 三个词被 Chat Highlight 染成金色。

Chat Highlight 的关键词默认是一组预设（如 `server, look here, important`）。设置按钮只能在几组预设间循环；想用任意关键词，直接编辑 `gamedata/horizon-mods/.horizon-mod-data/chathighlight/keywords` 文件（逗号分隔）。

---

## 三、包里有什么

```
START_HERE.md            本说明
START_GAME.cmd           一键启动（带九个模组）
PREPARE_RESOURCES.cmd    从你的 Minecraft 导入 26.3 资源
game/                    完整游戏目录
  horizon-client.exe     注入并编译好的客户端（模组已内置）
  horizon-ctl.exe        启动控制工具
  horizon-prepare.exe    资源导入工具
  dxcompiler.dll         渲染依赖
  gamedata/              配置、模组设置、存档（Ports Proof）
  gamelogs/              运行日志
  .local/horizon-client/26.3/   26.3 资源（待 PREPARE_RESOURCES 填充）
source/                  全部九个模组的 Rust 源码与单测
  whenchat-horizon/ chattimestamps-horizon/ timeformatter-horizon/
  ports/                 六个原创模组的注入源码
  native-v2/             新版 horizon-mod.toml + trait 格式工程
  tools/                 注入、构建、验证脚本
evidence/                真机截图与验证日志
```

---

## 四、必须知道的三个限制

1. **这不是官方 `.hnmod` 模组格式。** Horizon 的 `native-build` 需要 `nightly-2026-11-13-9f42064` 的官方 mod 工具链，而官方从未发布任何 `sdk-*` release。所以这里走的是等价路径：把模组源码注入真实 Horizon 源码后用本机 Rust 编译出完整客户端。`source/native-v2/` 里已经写好了正式格式工程，等工具链就绪可平移。

2. **启动器的 Play 不会带这些模组。** 模组是编译进这个特定 `horizon-client.exe` 的，必须用 `START_GAME.cmd` 启动。用启动器 Play 会回到没有模组的原版客户端。

3. **Durability、Auto Reconnect、Item Tooltip+ 只在注入形态下完整工作。** 官方 `Host` API 目前没有暴露 `item max_damage`，也没有连接服务器的方法，所以这几个模组依赖这些能力的部分在 trait 形态下无法实现。`Coordinates HUD`、`Chat Highlight`、`FPS HUD` 在新版格式下可以实现。

另外，游戏资源来自你自己的正版 Java 26.3；本包没有、也不会重新分发 Minecraft 的任何素材。

---

## 五、来源与许可

- 三个移植模组基于 ErikEdits/WhenChat、PlagueTR/ChatTimestamps、m3z0id/TimeFormatter，均为 MIT，版权与署名保留在各自目录的 `LICENSE` 和 `NOTICE.md`。
- 六个原创模组（坐标/耐久/重连/聊天高亮/FPS/物品提示）与整合脚本为本项目的产物。
- Horizon 主项目为 LGPL-3.0-or-later；分发编译后的二进制请注意相应义务。
- Horizon 是非官方粉丝项目，与 Mojang / Microsoft 无关联。

详细技术过程见 `source/README.md`、`source/WINDOWS-BUILD.md`、`source/TESTING.md`、`source/DELIVERY.md`。
