//! 用于定义可在vDSO编译单元和主编译单元间共享的数据结构。
//!
//! 这些数据结构分为两个类型，分别存放在两个子模块中：
//!
//! - shared：vDSO共享库的共享数据结构。在该子模块中，需要自己声明所需的数据结构后，将其全部放入已有的`VvarData`数据结构中（例如当前的`shared::SharedExample`）。
//! - argument：vDSO共享库的API函数参数和返回值数据结构。
//!
//! 这些数据结构均需要声明为`#[repr(C)]`。
//!
#![no_std]

pub mod argument;
pub mod shared;
