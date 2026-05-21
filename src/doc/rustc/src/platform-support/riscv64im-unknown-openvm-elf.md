# `riscv64im-unknown-openvm-elf`

**Tier: 3**

Target for [OpenVM](https://github.com/openvm-org/openvm/) virtual machines with the RV64IM ISA and custom RISC-V extensions defined through OpenVM's extension framework.

## Target maintainers

[@jonathanpwang](https://github.com/jonathanpwang)
[@yi-sun](https://github.com/yi-sun)

## Background

This target is an execution environment to produce a verifiable cryptographic proof of execution of
a RISC‑V ELF binary and any output that the developer wishes to make public.
The execution environment is implemented as a virtual machine in software only. The target is not intended for bare metal hardware. The virtual machine may be extended to support custom RISC-V instruction sets, which may be invoked from Rust via the `asm!` macro. See the [OpenVM Book] for further documentation on the architecture and usage.

We provide a cargo extension called [cargo-openvm] that provides tools for cross-compilation, execution, and generating cryptographic proofs of execution.

## Requirements

The target supports cross-compilation from any host and does not support host tools. It supports `alloc` with a
default allocator. Partial support for the Rust `std` library is provided using custom RISC-V instructions and requires the `openvm` crate with the `"std"` feature enabled.

`std` behaves as follows on this target:

- `std::io::stdout` / `std::io::stderr` (and `print!`/`eprintln!`/`println!`) write through `sys_write` to the host's stdout. UTF-8 is assumed.
- `std::io::stdin` is not supported — `sys_read` halts the VM with an `UNIMP` exit code. Programs should read inputs through `openvm::io::read` / `read_vec` instead.
- `std::env::var` / `std::env::vars` always return as if no environment variables are set (`sys_getenv` returns "not set").
- `std::env::args` is empty (`sys_argc` returns 0). Direct `sys_argv` calls halt the VM with `UNIMP`.
- `std::random` / `getrandom` are supplied from a deterministic host hint stream. The host can substitute any byte sequence; **this is not cryptographically secure** and must not be used for key material or anything else trust-dependent.
- `std::time::Instant::now()` and `std::time::SystemTime::now()` are deterministic placeholders: `Instant` is always `Duration::ZERO` and `SystemTime` is always `UNIX_EPOCH`. They do not panic but do not provide wall-clock time; `elapsed()` and `duration_since(earlier)` consistently return `Duration::ZERO`.
- `std::process` (spawn, exit codes, signals), `std::os` (most of it), `std::pipe`, `std::thread` (spawning), and other OS-shaped APIs are not implemented; their use generally halts the VM with `UNIMP`. Single-threaded code paths in `std::thread` (e.g. `current()`) still work.

Further commentary and the `openvm` crate's higher-level I/O helpers are documented in the [OpenVM book](https://docs.openvm.dev/book/writing-apps/writing-a-program#rust-std-library-support).

The target's execution environment is single-threaded, non-preemptive, and does not support
privileged instructions. At present, unaligned accesses are not supported and will result in execution traps. The binaries expect no operating system and can be thought
of as running on bare-metal. The target does not use `#[target_feature(...)]` or
`-C target-feature=` values.

Binaries are expected to be ELF.

Calling `extern "C"` uses the C calling convention outlined
in the [RISC-V specification].

## Building the target
The target can be built by enabling it for a `rustc` build.

```toml
[build]
target = ["riscv64im-unknown-openvm-elf"]
```

## Building Rust programs

Upstream Rust does not yet ship pre-compiled artifacts for this target. To compile for
this target, you will need to do one of the following:
- Build Rust with the target enabled (see "Building the target" above)
- Build your own copy of `core` by passing args `-Zbuild-std=alloc,core,proc_macro,panic_abort,std -Zbuild-std-features=compiler-builtins-mem`
- Use `cargo openvm build` provided by the cargo extension [cargo-openvm].
- Install the prebuilt OpenVM toolchain (see below).

### Prebuilt toolchain (recommended)

The OpenVM project publishes a forked rustc with this target built in, including prebuilt `std` rlibs for the host and the guest target, at <https://github.com/openvm-org/rust/releases>. The recommended install + build path is:

```sh
cargo openvm toolchain install                      # downloads + rustup-links the latest
cargo openvm build                                  # builds the guest crate with the right rustflags
```

`cargo openvm build` sets the user-crate rustflags described below (including `-Cpasses=lower-atomic`) and selects the linked openvm toolchain automatically; it is the supported entry point for guest programs.

If you must invoke `cargo` directly (e.g. when integrating into another build system), you have to pass the lower-atomic rustflag yourself or the linker will see unsupported A-extension instructions emitted from user code:

```sh
RUSTFLAGS="-Cpasses=lower-atomic" \
  cargo +<openvm-toolchain> build --target riscv64im-unknown-openvm-elf
```

The prebuilt `std` is already lowered at toolchain-build time, so `-Zbuild-std` is not needed in either case.

### OpenVM build flags

`cargo-openvm` calls `cargo` with `rustc` flags `-C passes=lower-atomic -C link-arg=-Ttext=<TEXT_START>` to map text to the appropriate location. The text start (presently `0x0020_0800`) must be set to start above the stack top, and heap begins right after the text. The utility also includes `--cfg getrandom_backend="custom"` to enable a custom backend for the `getrandom` crate and `--cfg openvm_intrinsics` to gate guest-specific codegen in the openvm runtime crates.

The target itself does **not** model the RISC-V A (atomic) extension. The OpenVM execution environment is single-threaded, so atomic operations are lowered to non-atomic ones by the `lower-atomic` LLVM pass at codegen time. See the "Prebuilt toolchain" section above for the corresponding build-command guidance.

## Testing

Note: this target is implemented as a software virtual machine; there is no hardware implementation.

Guest programs cross-compiled to the target must be run on the host inside OpenVM virtual machines, which are software emulators. The most practical way to do this is via either the [cargo-openvm] command-line interface or the [OpenVM SDK].

The target currently does not support running the Rust test suite.

## Cross-compilation toolchains and C code

Compatible C code can be built for this target on any compiler that has a RV64IM
target. On clang and ld.lld linker, it can be generated using the
`-march=rv64im`, `-mabi=lp64` with llvm features flag `features=+m` and llvm
target `riscv64-unknown-none`.

[RISC-V specification]: https://riscv.org/wp-content/uploads/2015/01/riscv-calling.pdf
[OpenVM]: https://github.com/openvm-org/openvm/
[OpenVM Book]: https://docs.openvm.dev/book/
[OpenVM SDK]: https://docs.openvm.dev/book/advanced-usage/sdk
[cargo-openvm]: https://docs.openvm.dev/book/getting-started/install
