# AGENTS.md

仓库整体的现状、目录地图、常用命令、架构约束、提交信息、文档同步与发版流程都在 [CLAUDE.md](CLAUDE.md)
（随本文件载入 `docs/contributing.md`）——**动手前先读它**，两份不要各写一套。本文件只写 fork 本地工作流，
不重复上游约定。

## 这个 fork 是干什么的

- 目的：在官方 `main` 之上提前拿到**已通过 CI 的 PR** 与本地自用改动，用 GitHub Actions 打 Windows 安装包
  （官方暂不包含未合并 PR，也没有签名版）。
- `personal = 官方 main + 已合绿 PR + 本地功能`，是唯一开发分支；本机 `main` 只跟踪官方，别在上面开发。
- 远程：`origin` = 官方 `qingjian-team/qingjian`，`fork` = `Schallophy/qingjian`；`personal` 跟踪 `fork/personal`。
- `.github/workflows/build-installer.yml`（push `personal` / 手动触发 → 上传安装包 artifact）与本文是 **fork 本地内容，
  不要给上游提 PR**；`ci.yml` / `release.yml` / `audit.yml` 是上游的，别改。

## 本地只做代码检查，编译打包交给 CI

改完在本地按 CI core job 的同一套跑（Windows 上先 `$env:QINGJIAN_UIACCESS='0'`，否则 Server 的测试二进制
带 uiAccess 清单起不来，报 os error 740）：

```powershell
cargo fmt --all --check
cargo clippy --workspace --exclude qingjian-macos --all-targets --locked -- -D warnings
cargo test  --workspace --exclude qingjian-macos --locked
```

- `--exclude qingjian-macos` 必须带上：那是 AppKit 壳，Windows 上编不过；这与 CI core job 一致。
- **不要**本地 `cargo build --release`、打 Inno 安装包、装到本机——那是 CI 的活（见下）。

## 拉上游更新 + 合绿 PR

**每个 PR 合之前至少本地审一遍**（`gh pr diff` 或 `git show pr-N`）：确认改动在范围内、没碰不该碰的平台/分支、
没引入上游明确拒收的东西（主题与自绘渲染器 `qingjian-render` 暂不接受 PR）。

```powershell
git switch personal
git fetch origin --tags --prune
git merge --no-ff --no-edit origin/main                      # 官方新提交

# 每个候选 PR：先看状态与 diff，再合
gh pr view <N> -R qingjian-team/qingjian --json state,mergeable,statusCheckRollup
gh pr diff <N> -R qingjian-team/qingjian
git fetch origin pull/<N>/head:pr-<N>
git merge --no-ff --no-edit pr-<N>                           # 有冲突就地解（多为并集），别 abort

git push --no-verify fork personal                           # 触发 CI 出包
```

- 已合了哪些 PR：`git log --merges --oneline origin/main..personal`（形如 `Merge branch 'pr-N' into personal`）。
- 某个 PR 被上游合进 `main` 后**不要再合它的 `pr-N` 分支**，改用 `git merge origin/main` 带进来，否则重复改动冲突。
- 冲突高发：`docs/`（README / preferences / crate-notes）与共享的 `crates/qingjian-platform` 配置。
- `.githooks` 的 pre-commit 会跑全 workspace 的 fmt + clippy，pre-push 会跑全测试，整合期很慢：
  合并提交用 `git -c core.hooksPath=.git/no-hooks merge --no-verify ...` 跳过，**自己改完代码后再单独跑上面的检查**。

## 出包与安装（CI 做）

```powershell
gh run list -R Schallophy/qingjian --workflow build-installer --limit 3
gh run watch <run-id> -R Schallophy/qingjian --exit-status
gh run download <run-id> -R Schallophy/qingjian -n Qingjian-Setup -D C:\Users\Schallophy\projects\setup
```

直接装即覆盖升级（同 `C:\Program Files\Qingjian`）。安装器会杀/起 Server，但 TSF DLL 在**每个已打开的应用进程**里，
只有新开的程序才用新 DLL。包无签名：SmartScreen 需点「仍要运行」，任务栏搜索等 UWP 界面候选窗可能被盖。
