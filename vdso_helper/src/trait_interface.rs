//! 生成vDSO的依赖接口

/// 通过trait声明定义vDSO的依赖接口。
///
/// 在该宏中定义一个trait接口时，会在保留trait定义的同时自动生成一个对应的vtable表（`$name_TABLE`）、一个vtable初始化函数（`init_vtable_$name`）和一个虚拟实现结构体（`$nameVirtImpl`）。
///
/// 从而，外部可以提供对这个trait接口的实现，并通过初始化函数注册到vDSO的vtable中。外部通过指针形式将实现了该trait接口的结构体传递给vDSO。
///
/// vDSO中的函数实现则可以将指向泛型的指针转化为`$nameVirtImpl`的引用后，通过虚拟实现结构体调用这些trait中的函数，而不需要直接操作函数指针。
///
/// 这样声明的接口应该放在`interface.rs`中，这样`build_vdso`就会为`init_vtable_$name`函数生成调用vDSO内函数的接口，就像其为`api.rs`中的函数生成接口一样。
#[macro_export]
macro_rules! trait_interface {
    ($(#[doc = $trait_doc:literal])* pub trait $name:ident { $($(#[doc = $fn_doc:literal])* fn $fn_name:ident $args:tt $(-> $ret:ty)?;)+ }) => {
        $(#[doc = $trait_doc])*
        pub trait $name {
            $(
                $(#[doc = $fn_doc])*
                fn $fn_name $args $(-> $ret)?;
            )+
        }

        $crate::paste::paste! {
            #[allow(missing_docs)]
            pub(crate) static [<$name _TABLE>]: $crate::lazyinit::LazyInit<[usize; $crate::count!($($fn_name)+)]> = $crate::lazyinit::LazyInit::new();
        }

        $crate::paste::paste! {
            #[repr(usize)]
            #[allow(missing_docs)]
            pub(crate) enum [<$name _FnIndex>] {
                $($fn_name),+
            }
        }

        $crate::paste::paste! {
            #[unsafe(no_mangle)]
            #[allow(missing_docs)]
            pub extern "C" fn [<init_vtable_ $name>]($($fn_name : usize),+) {
                [<$name _TABLE>].init_once([$($fn_name),+]);
            }
        }

        $crate::paste::paste! {
            #[allow(missing_docs)]
            #[derive(Debug)]
            pub(crate) struct [<$name VirtImpl>];
        }

        $crate::paste::paste! {
            #[allow(missing_docs)]
            impl [<$name VirtImpl>] {
                pub unsafe fn from_ptr(ptr: *const ()) -> &'static Self {
                    unsafe { &*(ptr as *const Self) }
                }

                pub unsafe fn from_mut(ptr: *mut ()) -> &'static mut Self {
                    unsafe { &mut *(ptr as *mut Self) }
                }

                pub fn to_ptr(&'static self) -> *const () {
                    self as *const Self as _
                }

                pub fn to_mut(&'static mut self) -> *mut () {
                    self as *mut Self as _
                }
            }
        }

        $crate::paste::paste! {
            #[allow(missing_docs)]
            impl $name for [<$name VirtImpl>] {
                $(
                    fn $fn_name $args $(-> $ret)? {
                        let f: $crate::fn_ptr_type!($args $(-> $ret)?) = unsafe { core::mem::transmute([<$name _TABLE>][[<$name _FnIndex>]::$fn_name as usize]) };
                        $crate::fn_call!(f $args)
                    }
                )+
            }
        }
    };
}

/// 计算trait中函数的数量的宏。
#[macro_export]
macro_rules! count {
    ($f1:ident $($f2:ident)*) => {
        (1 + $crate::count!($($f2)*))
    };
    () => {
        0
    };
}

/// 将函数的参数列表转化为函数类型
#[macro_export]
macro_rules! fn_ptr_type {
    ((&self $(,$arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?) => {
        fn(&Self $(, $arg_ty)*) $(-> $ret)?
    };
    ((&mut self $(,$arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?) => {
        fn(&mut Self $(, $arg_ty)*) $(-> $ret)?
    };
    (($($arg:ident: $arg_ty:ty),*) $(-> $ret:ty)?) => {
        fn($($arg_ty),*) $(-> $ret)?
    };
}

/// 将函数的参数列表转化为函数调用时的参数列表
#[macro_export]
macro_rules! fn_call {
    ($fn_name:ident (&$self:ident $(, $arg:ident: $arg_ty:ty)*)) => {
        $fn_name($self $(, $arg)*)
    };
    ($fn_name:ident (&mut $self:ident $(, $arg:ident: $arg_ty:ty)*)) => {
        $fn_name($self $(, $arg)*)
    };
    ($fn_name:ident ($($arg:ident: $arg_ty:ty),*)) => {
        $fn_name($($arg),*)
    };
}
