use core::sync::atomic::Ordering;
pub use core::task::{Poll, Context};
use core::{mem::MaybeUninit};

use vdso_helper::{async_api, get_vvar_data, log};

use crate::{interface, ArgumentExample, PRIVATE_DATA_EXAMPLE};

#[unsafe(no_mangle)]
pub extern "C" fn get_shared() -> ArgumentExample {
    ArgumentExample {
        i: get_vvar_data!(example).load(Ordering::Acquire),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_shared(i: usize) {
    get_vvar_data!(example).store(i, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn get_private() -> ArgumentExample {
    ArgumentExample {
        i: PRIVATE_DATA_EXAMPLE.load(Ordering::Acquire),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_private(i: usize) {
    PRIVATE_DATA_EXAMPLE.store(i, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn test_args(
    a: Option<usize>,
    b: Result<usize, ()>,
    c: (usize, usize),
) -> (Option<usize>, Result<usize, ()>, (usize, usize)) {
    (a.map(|i| i + 1), b.map(|i| i + 1), (c.0 + 1, c.1 + 1))
}

#[unsafe(no_mangle)]
pub extern "C" fn test_call(ptr: *mut ()) {
    interface::test_call(ptr);
}

#[unsafe(no_mangle)]
pub extern "C" fn test_log() {
    log::error!("Hello, this is a log within the vDSO!");
    log::warn!("Hello, this is a log within the vDSO!");
    log::info!("Hello, this is a log within the vDSO!");
    log::debug!("Hello, this is a log within the vDSO!");
    log::trace!("Hello, this is a log within the vDSO!");
}

// #[unsafe(no_mangle)]
// pub extern "C" fn init_TestIf_table(test_fn1: usize, test_fn2: usize, test_fn3: usize) {
//     interface::TestIf_TABLE.init_once([test_fn1, test_fn2, test_fn3]);
// }

// ---build_vdso的异步api部分需要读取的内容---

#[repr(C)]
pub struct TestYieldFuture {
    arg: usize,
    yielded: bool,
}

impl TestYieldFuture {
    pub fn new(arg: usize) -> Self {
        Self {
            arg,
            yielded: false,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn test_yield_poll(fut: *mut TestYieldFuture, _cx: &mut Context<'_>) -> Poll<()> {
    if unsafe { (*fut).yielded } {
        Poll::Ready(())
    } else {
        unsafe { (*fut).yielded = true };
        Poll::Pending
    }
}

async_api!(test_yield, TestYieldFuture, test_yield_poll);

// ---build_vdso的异步api部分需要生成的内容---

// #[repr(transparent)]
// pub struct TestYieldFutureWrapper(TestYieldFuture);

// impl Future for TestYieldFutureWrapper {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         unsafe {
//             test_yield_poll(
//                 core::mem::transmute::<*mut TestYieldFutureWrapper, *mut TestYieldFuture>(
//                     self.get_mut(),
//                 ),
//                 cx,
//             )
//         }
//     }
// }

// pub async fn test_yield(arg: usize) -> () {
//     TestYieldFutureWrapper(TestYieldFuture::new(arg)).await
// }
