//! 用于调用vDSO共享库的API。
//!
//! 提供的API分为两个部分：用于初始化该模块的`init_vdso_vtable`函数，和vDSO共享库的API。
//!
//! 该模块的函数由`build.rs`脚本自动生成，用户不需改动。
//!
//! ## `init_vdso_vtable`
//!
//! 其声明为：`pub unsafe fn init_vdso_vtable(base: u64, vdso_elf: &ElfFile);`。
//!
//! 其传入参数分别为vDSO共享库的基地址和其ELF文件。其会从ELF文件中读取vDSO共享库的API函数地址并存储，以用于后续调用。
//!
//! 只有在调用了该函数后，才能使用本模块提供的其它API函数。
//!
//! ## vDSO共享库的API
//!
//! 该模块会生成与vDSO共享库的API同名、相同声明的函数。
//!
//! 调用这些函数后，该模块会读取已存储的对应函数地址，并调用vDSO共享库内部的相应函数。
//!
#![no_std]

#[rustfmt::skip]
mod apis {
    include!(concat!(env!("OUT_DIR"), "/api.rs"));
}

pub use apis::*;
