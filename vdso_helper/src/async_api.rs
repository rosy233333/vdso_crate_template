//! 辅助build_vdso组装async接口。

/// 目前这个宏不会产生代码，而是用作build_vdso读取的标记。
///
/// build_vdso读取该宏的三个参数，提取对应的内容，组装成一个异步API。
///
/// ## vDSO异步API的构建
///
/// 为了构建vDSO的异步API，需要用户在`api.rs`中提供以下内容：
///
/// 1. 一个数据结构，用于封装异步的状态。
///     - 除了异步逻辑中所需的状态以外，异步API的参数也属于状态，需要存储到状态数据结构中。
///     - 需要实现`new`方法，根据一些参数创建状态数据结构的实例（`Self`）。
///     - `new`方法的参数即为生成的异步API的参数。
///     - 该数据结构不以API形式提供，因此无法访问vDSO中的共享和私有数据。
/// 2. 以vDSO同步API形式（也就是`#[no_mangle]\npub extern "C" fn 函数名(参数) -> 返回值`）提供的poll函数。
///     - 因为是API形式，所以可以访问vDSO中的共享和私有数据。
///     - 参数表：
///         - #0: 状态数据结构的可变裸指针（`*mut State`）
///         - #1: 上下文的可变引用（`&mut Context<'_>`）
///     - 返回值：`Poll<RetType>`，其中RetType为异步API的返回值。
/// 3. 调用`async_api`宏，分别传入以下三个参数：
///     - #0: 生成的异步API的名称
///     - #1: 状态数据结构的名称
///     - #2: poll函数的名称
///
/// ## 示例
///
/// `api.rs`中提供的内容：
///
/// ```Rust
/// #[repr(C)]
/// pub struct TestYieldFuture {
///     arg: usize,
///     yielded: bool,
/// }
///
/// impl TestYieldFuture {
///     pub fn new(arg: usize) -> Self {
///         Self {
///             arg,
///             yielded: false,
///         }
///     }
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn test_yield_poll(fut: *mut TestYieldFuture, _cx: &mut Context<'_>) -> Poll<()> {
///     if unsafe { (*fut).yielded } {
///         Poll::Ready(())
///     } else {
///         unsafe { (*fut).yielded = true };
///         Poll::Pending
///     }
/// }
///
/// async_api!(test_yield, TestYieldFuture, test_yield_poll);
/// ```
///
/// `build_vdso`生成的内容：
///
/// ```Rust
/// #[repr(transparent)]
/// pub struct TestYieldFutureWrapper(TestYieldFuture);
///
/// impl core::future::Future for TestYieldFutureWrapper {
///     type Output = ();
///
///     fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
///         unsafe {
///             test_yield_poll(
///                 core::mem::transmute::<*mut TestYieldFutureWrapper, *mut TestYieldFuture>(
///                     self.get_mut(),
///                 ),
///                 cx,
///             )
///         }
///     }
/// }
///
/// /// 最终生成的异步API
/// pub async fn test_yield(arg: usize) -> () {
///     TestYieldFutureWrapper(TestYieldFuture::new(arg)).await
/// }
/// ```
#[macro_export]
macro_rules! async_api {
    ($async_fn_name:ident, $fut_struct_name:ident, $poll_fn_name:ident) => {};
}
