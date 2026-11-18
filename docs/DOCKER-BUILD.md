# 用 Docker/Linux 做整包编译与实测 — 可行性与正确流程

最后更新：2026-11-17（深夜）

> **最终状态：本文方案已实际跑通。** 容器 `hn-build` 里 `native.py check all` 三模组全绿（真实 Horizon `9ee13cd0` + rustc 1.95）。实践中踩到并解决的坑（本文下方早期估算未覆盖）：
> - 容器 Config.Env 写死失效代理 `60145`，cargo 全部要 `env -u` 清代理走直连。
> - cargo 依赖用 **rsproxy 国内镜像**（`sparse+https://rsproxy.cn/index/`，配在 `/usr/local/cargo/config.toml`），避开 crates.io 大 index 文件经代理的 502。
> - Horizon 要求 **rustc ≥1.95**（非 1.90）；rustc 组件 80MB 从**华为云镜像** `mirrors.huaweicloud.com/rustup/dist/...` 断点续传下载，按 sha256 命名放入 `/usr/local/rustup/downloads/` 后 `rustup toolchain install 1.95.0` 成功。
> - apt 系统库（alsa/libudev/wayland/x11 等 dev 包）换华为云 Debian 源安装，默认源清代理后太慢会卡死。
> - 详见工作日志 `.workbuddy/memory/2026-11-17.md`。下方保留最初的方案说明。

## 一、先纠正三个关键误解

你问了三个问题，逐一回答（基于 `native.py` 源码逐行核实）：

### Q1：光把 Horizon 源码下下来，是不是就能测了？—— 不是

`native.py check` 的机制里，**Horizon 源码是你不用管的**：它会自己从 `codeload.github.com` 下载 pinned commit 的 tar.gz 并解压到 `native/target/horizon/<rev>/`。你手动下 Horizon 源码反而用不上。

真正需要的是三样东西，缺一不可：

1. **`horizon-mods` 仓库**（提供 `native/tools/native.py` 这个工具本身）
2. **`horizon-mixins` CLI**（native.py 用 `cargo install --git` 自动装，也无需手动）
3. **把我们的三个模组放进 `horizon-mods/native/<id>/` 正确目录结构**（这是你需要做的适配）
4. 工具链：**Rust 1.90+**（本机 1.100 ✅）、**Python 3.11+ with tomllib**（本机 3.13 ✅）、**rsync**（本机 Windows 缺）

`native.py check` 实际做的事：
```
codeload 下 Horizon 源码 → 检查 hook 版本 → cargo install horizon-mixins CLI
→ mixins apply 把模组注入 Horizon → rsync 同步 → cargo check -p horizon-client（编译整个游戏）
```
其中 `cargo check` 是重头，会拉几百个 Rust 依赖并编译整个游戏客户端。

### Q2：Docker 上能不能弄 Linux 环境跑？—— 能，但本机当前有障碍

- Docker 客户端已装（v29.9.3），但 **Desktop 引擎没运行**，且本机 `wsl.exe` 被安全策略拉黑，我无法代你启动引擎（启动命令被安全策略拦截）。
- 即使引擎起来，容器里要能：访问 GitHub（codeload/api/github.com）、装 Rust 1.90+ / rsync / Python 3.11+。**网络刚才 502，现已恢复**（三域名都通）。
- Docker 方案理论可行，但你要自己先手动启动 Docker Desktop（双击桌面图标 / 开始菜单），并确保 WSL2 后端可用（可能被安全策略限制）。

### Q3：现在是不是就能实测了？—— 整包编译这步现在具备条件了

关键变化：**GitHub 网络已恢复**（codeload/api/github.com 全部 200/206）。这意味着 `native.py check` 的两大外部依赖（下载源码 + cargo install）都通了。本机 Rust 1.100、Python 3.13 满足要求。**唯一仍缺的是 Linux 环境**（native.py 用了 `tarfile filter="data"`、`rsync`、`cargo install --git`，在 Windows Git Bash 上跑不通）。

---

## 二、三条可行路径（按推荐顺序）

### 路径 A：Docker 容器（你倾向的方案）

需要你手动做的：
1. 启动 Docker Desktop（我这边被安全策略拦，无法代启动）。若 WSL2 后端因安全策略不可用，需在 Docker Desktop 设置里切到 Hyper-V 后端（如果用 Windows 10/11 Pro）。
2. 拉一个 Rust 官方镜像（自带 Rust + cargo）：
   ```sh
   docker pull rust:1.90-slim
   ```
3. 容器里补装 `rsync` 和 Python 3.11+：
   ```sh
   docker run -it --name horizon-build -v <本机horizon-ports路径>:/work -w /work rust:1.90-slim bash
   # 容器内：
   apt-get update && apt-get install -y rsync python3 git curl ca-certificates
   ```
4. 把模组放进 `horizon-mods/native/` 结构（见第三节），然后：
   ```sh
   cd /work/horizon-mods
   python3 native/tools/native.py check whenchat chattimestamps timeformatter
   ```

### 路径 B：WSL2 原生 Ubuntu（更稳，但 wsl.exe 被安全策略拦）

需要先在「安全中心 → 命令安全 → 程序黑名单」把 `wsl.exe` 放行，然后 `wsl --install -d Ubuntu-24.04`。本机安全策略放行后我可继续。

### 路径 C：GitHub Actions 云端（完全绕开本机环境，最省事）

`horizon-mods` 仓库自带 `.github/workflows/native.yml`，PR 时自动跑整包编译。把模组按结构放进 fork 的仓库，开 PR，GitHub 云端的 Linux 环境会跑 `native.py check` 并把报告贴到 PR 检查里。**不需要本机 Linux，不需要 Docker，不需要下载源码。**

---

## 三、模组必须放进的正确目录结构

`native.py` 要求模组在 `horizon-mods/native/<id>/`，且目录下有 `compatibility.json`，`mixins/` 里有 `horizon-mixin.toml`。当前我们的模组目录名带 `-horizon` 后缀、且 `mixins/` 下多一层，需要适配。三个模组的 id 已确认：`whenchat` / `chattimestamps` / `timeformatter`。

目标结构（以 whenchat 为例）：
```
horizon-mods/native/whenchat/
├── compatibility.json        # 已有，直接复制
├── LICENSE                   # 已有（native.py package 会用到）
├── mixins/whenchat/
│   ├── horizon-mixin.toml    # 已有
│   ├── runtime.rs            # 已有
│   ├── chat.rs / list.rs     # 已有
│   └── ...
└── (verify/ 是本机测试用，不属于 native mod 结构，可不放)
```

> 注意：`native.py package` 只打包 bundle 里的 `.rs`/`.toml` 和 LICENSE，verify 工程不会进 `.hnmod`，所以不放没问题。

---

## 四、下载链接（你要求的手工下载）

如果你仍想手动下源码看看：

| 内容 | 链接 | 说明 |
|---|---|---|
| Horizon 源码（pinned commit） | https://github.com/horizon-rust/horizon/archive/9ee13cd038559db05de6a4ced4496028ce409564.tar.gz | native.py 会自动下，**你无需手动下**；只是想看源码可用这个。注意：原 `85b5bcca` 是重构版（mod-api-gen），与模组注入点不兼容，已修正为 `9ee13cd0`（hooks 645）|
| Horizon 主仓 | https://github.com/horizon-rust/horizon | 浏览用 |
| horizon-mods 仓库（含 native.py） | https://github.com/horizon-rust/horizon-mods | 整包编译真正需要的仓库 |
| horizon-mixins 仓库 | https://github.com/horizon-rust/horizon-mixins | CLI 工具，native.py 自动装 |
| 官方 Windows 工具包 | https://github.com/horizon-rust/horizon/releases/latest 里的 `horizon-windows-x64.zip` | 含 horizon-client / horizon-prepare / 启动器，真机实测用 |

再次强调：**光下第一个（Horizon 源码）不够**，整包编译需要的是 `horizon-mods` 仓库 + 正确目录结构 + Linux 环境。

---

## 五、下一步建议

网络已恢复，整包编译的外部条件齐了。推荐：

1. **最快验证**：走路径 C（GitHub Actions），我在你 fork 的 horizon-mods 里按结构放好模组，开 PR 触发云端整包编译，报告直接可见，完全绕开本机 Docker/WSL 限制。
2. **本地 Docker**：你先手动启动 Docker Desktop + 确认 WSL2 后端可用，我再把容器方案和目录适配脚本写好。
3. 无论哪条，`native.py check` 通过后才能用 `package` 出 `.hnmod`、再进游戏实测（实测清单见 TESTING.md）。

需要我先把模组适配成 `horizon-mods/native/<id>/` 结构并写一个一键脚本吗？
