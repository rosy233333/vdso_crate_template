//! 初始化日志模块

use core::sync::atomic::{AtomicUsize, Ordering};

use lazyinit::LazyInit;
use log::Log;

// /// *const 主编译单元中的Logger全局变量
// static LOGGER: AtomicUsize = AtomicUsize::new(0);
// /// fn enabled(&Logger, &Metadata) -> bool
// static ENABLED: AtomicUsize = AtomicUsize::new(0);
// /// fn log(&Logger, &Record)
// static LOG: AtomicUsize = AtomicUsize::new(0);
// /// fn flush(&Logger)
// static FLUSH: AtomicUsize = AtomicUsize::new(0);

static LOGGER: LazyInit<u128> = LazyInit::new();

/// 初始化vdso中的log。
///
/// 用户不需手动调用此函数，此函数会在初始化vdso时自动调用。
#[no_mangle]
pub extern "C" fn init_log(logger_fat_ptr: u128) {
    LOGGER.init_once(logger_fat_ptr);

    log::set_logger(&LOGGER_VIRT_IMPL).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

struct LogVirtImpl;
static LOGGER_VIRT_IMPL: LogVirtImpl = LogVirtImpl;

impl log::Log for LogVirtImpl {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // let enabled_ptr = ENABLED.load(Ordering::Acquire);
        // assert!(enabled_ptr != 0);
        // let logger_ptr = LOGGER.load(Ordering::Acquire);
        // assert!(logger_ptr != 0);
        // let f: fn(*const (), &log::Metadata<'_>) -> bool =
        //     unsafe { core::mem::transmute(enabled_ptr) };
        // f(logger_ptr as _, metadata)
        let logger: &'static dyn Log = unsafe { core::mem::transmute(*LOGGER) };
        logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        // let log_ptr = LOG.load(Ordering::Acquire);
        // assert!(log_ptr != 0);
        // let logger_ptr = LOGGER.load(Ordering::Acquire);
        // assert!(logger_ptr != 0);
        // let f: fn(*const (), &log::Record<'_>) = unsafe { core::mem::transmute(log_ptr) };
        // f(logger_ptr as _, record)
        let logger: &'static dyn Log = unsafe { core::mem::transmute(*LOGGER) };
        logger.log(record)
    }

    fn flush(&self) {
        // let flush_ptr = FLUSH.load(Ordering::Acquire);
        // assert!(flush_ptr != 0);
        // let logger_ptr = LOGGER.load(Ordering::Acquire);
        // assert!(logger_ptr != 0);
        // let f: fn(*const ()) = unsafe { core::mem::transmute(flush_ptr) };
        // f(logger_ptr as _)
        let logger: &'static dyn Log = unsafe { core::mem::transmute(*LOGGER) };
        logger.flush()
    }
}
