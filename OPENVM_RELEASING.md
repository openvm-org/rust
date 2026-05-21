# OpenVM Rust Toolchain — Release Process

This fork adds the `riscv64im-unknown-openvm-elf` tier 3 target and ships prebuilt toolchain tarballs as GitHub Releases.

## Branches

- `master` — tracks upstream `rust-lang/rust`. Never modified by us.
- `feat/riscv64im-unknown-openvm-elf` — the **source diff**: a single squashed commit on top of `master` with all openvm-specific changes (target spec, std PAL, panic_abort, bootstrap workaround, docs, the two GitHub Actions workflows, this file).
- `openvm-<channel>-<version>` — one branch per release, sitting on the exact upstream commit that backs that channel/date, with the source diff cherry-picked on top as a single commit. Tagged with the same name. Independent of `master` — `master` can keep advancing without disturbing existing release branches.

## Remotes

```sh
git remote add origin   git@github.com:openvm-org/rust.git
git remote add upstream https://github.com/rust-lang/rust.git
```
All `git push` below means `git push origin`.

## Naming

Toolchain name = release tag = release branch name (all the same string).

| Upstream channel  | OpenVM name                   |
|-------------------|-------------------------------|
| Stable `X.Y.Z`    | `openvm-X.Y.Z`                |
| Beta `YYYY-MM-DD` | `openvm-beta-YYYY-MM-DD`      |
| Nightly `YYYY-MM-DD` | `openvm-nightly-YYYY-MM-DD` |

`release-toolchain.yml` triggers on any tag matching `openvm-*`.

## Cutting a release

```sh
NEW=openvm-nightly-2026-08-01

# Find the upstream commit for that nightly:
NEW_BASE=$(curl -s https://static.rust-lang.org/dist/2026-08-01/channel-rust-nightly-git-commit-hash.txt)
# For stable, use the upstream tag (e.g. `1.91.0`). For beta, swap
# `nightly` → `beta` in the URL above.

git fetch upstream
git cat-file -e "$NEW_BASE" || { echo "commit not in local history"; exit 1; }
git checkout -b "$NEW" "$NEW_BASE"
git cherry-pick feat/riscv64im-unknown-openvm-elf
# Resolve any conflicts (most common: library/std/src/sys/io/error/mod.rs
# when upstream added a new target_os entry). `git add` + `git cherry-pick --continue`.
git tag -a "$NEW" -m "openvm rust toolchain for $NEW"
git push origin "$NEW"                # branch push only — does NOT fire the workflow
# Bump DEFAULT_RUSTUP_TOOLCHAIN_NAME in the downstream consumer's
# crates/toolchain/build/src/lib.rs to match $NEW. Open a PR there.
git push origin "refs/tags/$NEW"      # tag push — fires release-toolchain.yml
```

## What the workflow does

`release-toolchain.yml` (tag push or `workflow_dispatch`):
1. Three parallel build jobs via reusable `build-toolchain.yml`:
   | Host triple                 | Runner             |
   |-----------------------------|--------------------|
   | `x86_64-unknown-linux-gnu`  | `ubuntu-22.04`     |
   | `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` |
   | `aarch64-apple-darwin`      | `macos-14`         |
2. Each runs `python3 x.py build --stage 2 compiler/rustc library --target $HOST,riscv64im-unknown-openvm-elf`, then runs the **atomic smoke test** (`llvm-objdump -d` on every `riscv64im-unknown-openvm-elf` rlib, grepping for `lr.w/d`, `sc.w/d`, `amo*.w/d` — any hit fails the job and blocks release), then tars `build/$HOST/stage2/` (excluding `lib/rustlib/src` and `lib/rustlib/rustc-src`) as `rust-toolchain-$HOST.tar.gz`. ~2–3 h per job on hosted runners.
3. `release` job downloads all artifacts and runs `gh release create --draft`.

Tarballs ship prebuilt rlibs for both host and guest target — `lib/rustlib/<host>/lib/*.rlib` and `lib/rustlib/riscv64im-unknown-openvm-elf/lib/*.rlib`. The guest std is built with `-Cpasses=lower-atomic` (via `CARGO_TARGET_RISCV64IM_UNKNOWN_OPENVM_ELF_RUSTFLAGS` in the build job) and `panic = abort` (from the target spec), so consumers do **not** need `-Z build-std`. Both `rust-src` and `rustc-src` are excluded.

The downstream consumer invokes `cargo +<toolchain> build --target riscv64im-unknown-openvm-elf` and adds the openvm rustflags (`-Cpasses=lower-atomic`, `-C link-arg=-Ttext=…`, `--cfg getrandom_backend="custom"`, `--cfg openvm_intrinsics`) for its own user-crate codegen; std is already correctly compiled.

## Publishing the draft

After all three builds finish, open `https://github.com/openvm-org/rust/releases`, confirm the three `rust-toolchain-*.tar.gz` assets are attached, and hit "Publish release". Drafts are invisible to GitHub's public `latest` API; until you publish, the only way to consume the release is by explicit `--version <tag>`.

## Smoke-testing

```sh
cd /path/to/downstream-consumer
cargo install --path crates/cli         # rebuild with the new DEFAULT_RUSTUP_TOOLCHAIN_NAME
cargo openvm toolchain install          # downloads + rustup-links the latest release
cargo openvm toolchain list             # confirms it's linked + marked default
# In a guest crate:
cargo openvm build                      # builds for riscv64im-unknown-openvm-elf
# Cleanup if needed:
cargo openvm toolchain uninstall
```
While the release is still draft, pin the version: `cargo openvm toolchain install --version <tag>`.

## Rotating to a new upstream

```sh
git fetch upstream
git push origin master      # optional, keep our master synced
```
Then run "Cutting a release" with the new `NEW`. Previous release branches stay around — multiple toolchains coexist downstream under `~/.openvm/toolchains/`.

If upstream churn breaks the cherry-pick, rebase the source branch:

```sh
git checkout feat/riscv64im-unknown-openvm-elf
git rebase origin/master
git push --force-with-lease origin feat/riscv64im-unknown-openvm-elf
```

## File map (what's in the source diff)

- `compiler/rustc_target/src/spec/targets/riscv64im_unknown_openvm_elf.rs` — target spec.
- `compiler/rustc_target/src/spec/mod.rs` — target list entry + `Os::Openvm` variant.
- `library/std/src/sys/{alloc,args,env,pal/openvm,random,stdio,time}/openvm.rs` — std PAL stubs.
- `library/std/src/sys/{alloc,args,env,pal,random,stdio,time}/mod.rs`, `library/std/src/sys/io/error/mod.rs`, `library/std/src/sys/thread_local/mod.rs` — `cfg_select!` / cfg entries routing `target_os = "openvm"` to those PAL files.
- `library/std/src/sys/env_consts.rs` — `os::*` constants (FAMILY, EXE_EXTENSION, etc.) for the openvm target.
- `library/panic_abort/{Cargo.toml,src/lib.rs,src/openvm.rs}` — panic_abort PAL + alloc-dep gate.
- `library/std/{build.rs,src/random.rs}` — adds `openvm` to the no-libc target list and the random-source doc table.
- `library/test/src/{lib,console}.rs` — marks the `openvm` target as not supporting process spawning or threads.
- `src/bootstrap/src/lib.rs` — enables the `compiler-builtins-mem` feature when building `std` for the `openvm` target. The bootstrap *target-sanity-check workaround* is **not** in source — it lives in the build workflow, which writes a dummy `riscv64im-unknown-openvm-elf.json` and exports `RUST_TARGET_PATH` so stage-0 rustc can validate the triple before the patched compiler is built.
- `src/doc/rustc/src/{platform-support.md,SUMMARY.md,platform-support/riscv64im-unknown-openvm-elf.md}` — docs index + platform-support page.
- `tests/assembly-llvm/targets/targets-elf.rs` — new target revision in the assembly target-coverage test.
- `tests/ui/check-cfg/{cfg-crate-features,well-known-values}.stderr` — `expected values for target_os` lists, regenerated.
- `.github/workflows/{build,release}-toolchain.yml` — release pipeline.
- `OPENVM_RELEASING.md` — this file.

## Troubleshooting

- **"no space left on device" on Linux runners.** Stage-2 rustc is tight on `ubuntu-22.04*`. The workflow already runs a "Free disk space" step; if it's still not enough, switch to a larger hosted SKU (`ubuntu-22.04-large`) or a self-hosted/RunsOn runner with `disk=large`.
- **Tag push didn't trigger the workflow.** Confirm the tag matches `openvm-*` and that the release branch contains `.github/workflows/release-toolchain.yml`. Tags pushed before the workflow file existed in the branch won't fire — delete and re-push.
- **`rust-toolchain-<host>.tar.gz` 404s for one host.** That host's build job silently failed, or the consumer's host isn't one of the four we support. Check `https://api.github.com/repos/openvm-org/rust/releases/tags/<tag>`.
- **Cherry-pick conflict.** Almost always `library/std/src/sys/io/error/mod.rs` — upstream added a new `target_os` entry between the source branch's master and this release branch's upstream commit. Manually merge the new upstream entry with our `target_os = "openvm"` line.
