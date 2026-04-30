use crate::spec::{
    Arch, Cc, LinkerFlavor, Lld, LlvmAbi, Os, PanicStrategy, RelocModel, Target, TargetMetadata,
    TargetOptions,
};

pub(crate) fn target() -> Target {
    Target {
        data_layout: "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128".into(),
        llvm_target: "riscv64".into(),
        metadata: TargetMetadata {
            description: Some("OpenVM zero-knowledge Virtual Machine (RV64IM ISA)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true),
        },
        pointer_width: 64,
        arch: Arch::RiscV64,

        options: TargetOptions {
            os: Os::Openvm,
            vendor: "unknown".into(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),
            cpu: "generic-rv64".into(),

            // We set atomic_width to 64 for compatibility with crates such as crossbeam,
            // but this should never be triggered since compilation should always lower
            // atomics and be single-threaded.
            max_atomic_width: Some(64),
            atomic_cas: true,

            features: "+m".into(),
            llvm_abiname: LlvmAbi::Lp64,
            panic_strategy: PanicStrategy::Abort,
            relocation_model: RelocModel::Static,
            emit_debug_gdb_scripts: false,
            eh_frame_header: false,
            singlethread: true,
            ..Default::default()
        },
    }
}
