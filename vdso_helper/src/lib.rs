//! 提供vDSO库相关的辅助功能。
//!
//! - [`mod@vvar_data`]模块用于声明和使用vVAR共享数据。
//! - [`mod@mut_cfg`]模块用于在编译期由环境变量指定的常量。

#![no_std]
#![deny(missing_docs)]

pub mod mut_cfg;
pub mod vvar_data;
