use crate::spec::{
    Arch, Cc, CodeModel, LinkerFlavor, Lld, Os, PanicStrategy, RelocModel, Target, TargetMetadata,
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

            // OpenVM is single-threaded and does not model the RISC-V A extension.
            // Atomic ops are lowered to plain loads/stores by LLVM's lower-atomic
            // pass before execution.
            max_atomic_width: Some(64),
            atomic_cas: true,

            features: "+m".into(),
            llvm_abiname: "lp64".into(),
            executables: true,
            panic_strategy: PanicStrategy::Abort,
            relocation_model: RelocModel::Static,
            code_model: Some(CodeModel::Medium),
            emit_debug_gdb_scripts: false,
            eh_frame_header: false,
            singlethread: true,
            ..Default::default()
        },
    }
}
