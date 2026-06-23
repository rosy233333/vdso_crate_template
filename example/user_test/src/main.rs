use std::{fmt::Arguments, mem};

// use crate::map::map_vdso;
// use libvdsoexample::{interface::TestIf, *};
use libvdsoexample::*;
use log::Log;
use memmap2::MmapMut;

// mod map;

struct MemImpl;

// #[crate_interface::impl_interface]
// impl MemIf for MemImpl {
//     #[doc = " 分配用于vDSO和vVAR的空间，返回指向首地址的指针。"]
//     #[doc = ""]
//     #[doc = " 若需要实现vDSO和vVAR在多地址空间的共享，则需要在分配时使这块空间可被共享。"]
//     fn alloc(size: usize) -> *mut u8 {
//         let mut map = MmapMut::map_anon(size).unwrap();
//         let ptr = map.as_mut_ptr();
//         mem::forget(map);
//         ptr
//     }

//     #[doc = " 从`alloc`返回的空间中，设置其中一块的访问权限。"]
//     #[doc = ""]
//     #[doc = " `flags`可能包含：READ、WRITE、EXECUTE、USER。"]
//     fn protect(addr: *mut u8, len: usize, flags: MappingFlags) {
//         let mut libc_flag = libc::PROT_READ;
//         if flags.contains(MappingFlags::EXECUTE) {
//             libc_flag |= libc::PROT_EXEC;
//         }
//         if flags.contains(MappingFlags::WRITE) {
//             libc_flag |= libc::PROT_WRITE;
//         }
//         unsafe {
//             if libc::mprotect(addr as _, len, libc_flag) == libc::MAP_FAILED as _ {
//                 panic!("vdso: mprotect res failed");
//             }
//         };
//     }
// }
#[crate_interface::impl_interface]
impl MemIf for MemImpl {
    #[doc = " 在地址空间中分配用于vDSO和vVAR的虚存区域（不需同时分配物理页面），返回指向首地址的指针。"]
    #[doc = " "]
    #[doc = " 保证size为build_vdso传入的config.page_size的整数倍。"]
    #[doc = " 要求返回的地址也为config.page_size的整数倍。"]
    fn valloc(_vspace: usize, size: usize) -> *mut u8 {
        let mut map = MmapMut::map_anon(size).unwrap();
        let ptr = map.as_mut_ptr();
        mem::forget(map);
        ptr
    }

    #[doc = " 分配多块用于vDSO和vVAR的连续物理页，返回`PhysPagePtr`。"]
    #[doc = " "]
    #[doc = " 保证size为build_vdso传入的config.page_size的整数倍。"]
    #[doc = ""]
    #[doc = " 若需要实现vDSO和vVAR在多地址空间的共享，则需要在分配时使这块空间可被共享（即，可被多次`map`）。"]
    fn ppage_alloc(_size: usize) -> PhysPagePtr {
        0
    }

    #[doc = " 从`alloc`返回的虚存区域中，映射其中一块到某个物理页面并设置权限。"]
    #[doc = " "]
    #[doc = " 被映射的物理页面可能和其它地址空间共享，也可能由这个地址空间独占。"]
    #[doc = " "]
    #[doc = " 保证vaddr对齐到build_vdso传入的config.page_size；len为config.page_size的整数倍。"]
    #[doc = ""]
    #[doc = " `flags`可能包含：READ、WRITE、EXECUTE、USER。"]
    fn map(
        _vspace: usize,
        vaddr: *mut u8,
        _ppage: PhysPagePtr,
        size: usize,
        flags: MappingFlags,
        _shared: bool,
    ) {
        let mut libc_flag = libc::PROT_READ;
        if flags.contains(MappingFlags::EXECUTE) {
            libc_flag |= libc::PROT_EXEC;
        }
        if flags.contains(MappingFlags::WRITE) {
            libc_flag |= libc::PROT_WRITE;
        }
        unsafe {
            if libc::mprotect(vaddr as _, size, libc_flag) == libc::MAP_FAILED as _ {
                panic!("vdso: mprotect res failed");
            }
        };
    }

    #[doc = " 重新设置已映射好的，虚拟首地址为`vspace`区域的权限。"]
    #[doc = " "]
    #[doc = " 保证vaddr对齐到build_vdso传入的config.page_size。"]
    fn change_protect(_vspace: usize, vaddr: *mut u8, size: usize, flags: MappingFlags) {
        let mut libc_flag = libc::PROT_READ;
        if flags.contains(MappingFlags::EXECUTE) {
            libc_flag |= libc::PROT_EXEC;
        }
        if flags.contains(MappingFlags::WRITE) {
            libc_flag |= libc::PROT_WRITE;
        }
        unsafe {
            if libc::mprotect(vaddr as _, size, libc_flag) == libc::MAP_FAILED as _ {
                panic!("vdso: mprotect res failed");
            }
        };
    }

    #[doc = " 获取`vspace`空间中`vaddr`地址对应的内核虚拟地址。"]
    #[doc = " （也就是当前代码可以直接访问的地址）"]
    fn get_kernel_vaddr(_vspace: usize, vaddr: *mut u8) -> *mut u8 {
        vaddr
    }

    #[doc = " 复制物理页指针，复制前后指向同一块物理页。复制后，参数和返回值对应的两个指针均需可用。"]
    #[doc = " "]
    #[doc = " 如果物理页使用RAII管理，则需调用其`clone`方法。"]
    #[doc = " "]
    #[doc = " 如果物理页不使用RAII管理，则可以直接返回参数。"]
    fn ppage_clone(_ppage: PhysPagePtr) -> PhysPagePtr {
        0
    }
}

struct TestImpl(usize);

impl TestIf for TestImpl {
    fn test_fn1(&self, arg: usize) -> usize {
        log::info!("test_fn1 called with arg: {}, self.0: {}", arg, self.0);
        self.0 + arg
    }

    fn test_fn2(&mut self, arg: usize) -> usize {
        log::info!("test_fn2 called with arg: {}, self.0: {}", arg, self.0);
        self.0 += arg;
        self.0
    }

    fn test_fn3(arg: usize) {
        log::info!("test_fn3 called with arg: {}", arg);
    }
}

fn main() {
    env_logger::init();
    log::info!("Starting VDSO test...");
    // let regions = load_and_init(0);
    load_and_init(0);
    // println!("vDSO and vVAR loaded with the following regions:");
    // for (i, (addr, size, flags)) in regions.iter().enumerate() {
    //     println!(
    //         "Region {}: Address = 0x{:016x}, Size = {}, Flags = {:?}",
    //         i, *addr as usize, size, flags
    //     );
    // }
    let example: ArgumentExample = get_shared();
    assert!(
        example.i == 0,
        "Expected get_shared() to return 0, got {}",
        example.i
    );
    set_shared(1);
    let example: ArgumentExample = get_shared();
    assert!(
        example.i == 1,
        "Expected get_shared() to return 1, got {}",
        example.i
    );
    let example: ArgumentExample = get_private();
    assert!(
        example.i == 0,
        "Expected get_shared() to return 1, got {}",
        example.i
    );
    set_private(1);
    let example: ArgumentExample = get_private();
    assert!(
        example.i == 1,
        "Expected get_shared() to return 1, got {}",
        example.i
    );

    assert_eq!(test_args(Some(1), Ok(2), (3, 4)), (Some(2), Ok(3), (4, 5)));

    init_vtable_TestIf::<TestImpl>();
    let mut test_impl = TestImpl(10);
    let ptr = &mut test_impl as *mut TestImpl as *mut ();
    test_call(ptr);
    test_log();
    println!("Test passed!");
}
