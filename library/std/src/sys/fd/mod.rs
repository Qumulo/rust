//! Platform-dependent file descriptor abstraction.

#![forbid(unsafe_op_in_unsafe_fn)]

cfg_select! {
    target_env = "qumulo" => {
        pub use crate::qumulo::fd::*;
    }
    all(any(target_family = "unix", target_os = "wasi"), not(target_env = "qumulo")) => {
        mod unix;
        pub use unix::*;
    }
    target_os = "hermit" => {
        mod hermit;
        pub use hermit::*;
    }
    target_os = "motor" => {
        mod motor;
        pub use motor::*;
    }
    all(target_vendor = "fortanix", target_env = "sgx") => {
        mod sgx;
        pub use sgx::*;
    }
    _ => {}
}
