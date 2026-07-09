use crate::spec::{
    base, Cc, FramePointer, LinkerFlavor, Lld, PanicStrategy, SanitizerSet, StackProbeType, Target,
};

pub(crate) fn target() -> Target {
    let mut base = base::linux::opts();
    base.env = "qumulo".into();
    base.cpu = "x86-64".into();
    base.max_atomic_width = Some(64);
    // Build binaries as shared objects, this is the only way we can run them
    // N.B. This also impacts building std for x86-64_qumulo, as it doesn't like undefined
    // references to the "rust_sys" bindings.
    base.add_pre_link_args(LinkerFlavor::Gnu(Cc::Yes, Lld::No), &["-m64", "--shared"]);
    base.stack_probes = StackProbeType::Inline;
    base.static_position_independent_executables = true;
    base.supported_sanitizers = SanitizerSet::ADDRESS
        | SanitizerSet::CFI
        | SanitizerSet::LEAK
        | SanitizerSet::MEMORY
        | SanitizerSet::THREAD;

    base.has_thread_local = false;
    base.panic_strategy = PanicStrategy::Abort;
    base.requires_uwtable = true;
    base.frame_pointer = FramePointer::Always;
    base.entry_name = "rust_main".into();

    let data_layout =
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

    Target {
        llvm_target: "x86_64-unknown-linux-gnu".into(),
        metadata: crate::spec::TargetMetadata {
            description: Some("64-bit Qumulo Linux (kernel 3.2+, glibc 2.17+)".into()),
            tier: Some(1),
            host_tools: Some(true),
            std: Some(true),
        },
        pointer_width: 64,
        data_layout: data_layout.into(),
        arch: "x86_64".into(),
        options: base,
    }
}
