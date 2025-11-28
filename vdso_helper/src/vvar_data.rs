/// 声明共享数据结构。
///
/// 使用方式：
///
/// - 类似于struct的定义，在大括号中声明每一项的名称和类型。
/// - 需要在crate的根模块下使用该宏。
/// - 在项目各处通过`get_vvar_data`宏获取共享数据结构的引用。
#[macro_export]
macro_rules! vvar_data {
    ($($i:ident: $t:ty),* $(,)?) => {
        #[derive(Default)]
        #[repr(C)]
        pub struct VvarData {
            $(pub $i: $t),*
        }

        trait VvarDataRequirements: Default + Sync {}
        impl VvarDataRequirements for VvarData {}
    };
}

/// 生成获取共享数据结构的引用的代码。
///
/// 参数：
///
/// - $1: 共享数据结构的字段名（在vvar_data宏中定义）。
/// - $2: 映射代码和数据段过程中的页面大小（usize类型）（可不填写，则默认为4K）。
///
/// 返回值：&'static T类型，代表对相应结构的引用
///
/// SAFETY：
///
/// 需要在生成的so文件中检查，函数get_code_base的偏移量要小于一页。
///
/// 否则，该宏获取到的引用地址会不正确。
#[macro_export]
macro_rules! get_vvar_data {
    ($i:ident, $e:expr) => {{
        let vvar_size = (core::mem::size_of::<crate::VvarData>() + ($e - 1)) & !($e - 1);
        let data_base = $crate::vvar_data::get_code_base($e) - vvar_size;
        let vvar_data_ref = unsafe { &*(data_base as *const crate::VvarData) };
        &(vvar_data_ref.$i)
    }};
    ($i:ident) => {{
        let vvar_size = (core::mem::size_of::<crate::VvarData>() + (0x1000 - 1)) & !(0x1000 - 1);
        let data_base = $crate::vvar_data::get_code_base(0x1000) - vvar_size;
        let vvar_data_ref = unsafe { &*(data_base as *const crate::VvarData) };
        &(vvar_data_ref.$i)
    }};
}

/// 此处的pub仅用于在动态符号表中得到该函数的地址以便检查
///
/// 该函数不应被用户直接调用
#[inline(never)]
#[no_mangle]
#[link_section = ".text.start"]
pub fn get_code_base(page_size: usize) -> usize {
    let pc = unsafe { hal::asm::get_pc() };
    pc & !(page_size - 1)
}
