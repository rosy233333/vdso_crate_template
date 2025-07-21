//! 这里的与 Vsched 相关的实现可以在 build 脚本中来自动化构建，而不是手动构建出来
#![no_std]

#[rustfmt::skip]
mod apis {
    include!(concat!(env!("OUT_DIR"), "/api.rs"));
}

pub use apis::*;
