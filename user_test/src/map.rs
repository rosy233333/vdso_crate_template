// Copied and modified from https://github.com/AsyncModules/vsched/blob/e19b572714a6931972f1428e42d43cc34bcf47f2/user_test/src/vsched.rs
use include_bytes_aligned::include_bytes_aligned;
use libvdso::VvarData;
use memmap2::MmapMut;
use page_table_entry::MappingFlags;
use std::ptr::copy_nonoverlapping;
use std::str::from_utf8;
use xmas_elf::program::SegmentData;

const PAGES_SIZE_4K: usize = 0x1000;

const VVAR_SIZE: usize =
    (core::mem::size_of::<VvarData>() + PAGES_SIZE_4K - 1) & (!(PAGES_SIZE_4K - 1));
const VDSO: &[u8] = include_bytes_aligned!(8, "../../output/libvdso.so");
const VDSO_SIZE: usize =
    ((VDSO.len() + PAGES_SIZE_4K - 1) & (!(PAGES_SIZE_4K - 1))) + PAGES_SIZE_4K; // 额外加了一页，用于bss段等未出现在文件中的段

pub fn map_vdso() -> Result<MmapMut, ()> {
    let mut vdso_map = MmapMut::map_anon(VVAR_SIZE + VDSO_SIZE).unwrap();
    log::info!("vdso_map base: [{:p}, {:p}]", vdso_map.as_ptr(), unsafe {
        vdso_map.as_ptr().add(VVAR_SIZE + VDSO_SIZE)
    });
    log::debug!(
        "VVAR: VA:{:?}, {:#x}, {:?}",
        vdso_map.as_ptr(),
        core::mem::size_of::<VvarData>(),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
    );
    unsafe {
        if libc::mprotect(
            vdso_map.as_ptr() as _,
            core::mem::size_of::<VvarData>(),
            libc::PROT_READ | libc::PROT_WRITE,
        ) == libc::MAP_FAILED as _
        {
            log::error!("vvar: mprotect res failed");
            return Err(());
        }
    };
    let vvar = vdso_map.as_ptr() as *const u8 as *mut u8 as *mut () as *mut VvarData;
    unsafe { vvar.write(VvarData::default()) };

    let vdso_so = &mut vdso_map[VVAR_SIZE..];
    // #[allow(const_item_mutation)]
    // VDSO.read(vdso_so).unwrap();

    let vdso_elf = xmas_elf::ElfFile::new(VDSO).expect("Error parsing app ELF file.");
    if let Some(interp) = vdso_elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    {
        let interp = match interp.get_data(&vdso_elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("Invalid data in Interp Elf Program Header"),
        };

        let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
        // remove trailing '\0'
        let _interp_path = interp_path.trim_matches(char::from(0)).to_string();
        log::debug!("Interpreter path: {:?}", _interp_path);
    }
    let elf_base_addr = Some(vdso_so.as_ptr() as usize);
    let segments = elf_parser::get_elf_segments(&vdso_elf, elf_base_addr);
    let relocate_pairs = elf_parser::get_relocate_pairs(&vdso_elf, elf_base_addr);
    for segment in segments {
        if segment.size == 0 {
            log::warn!(
                "Segment with size 0 found, skipping: {:?}, {:#x}, {:?}",
                segment.vaddr,
                segment.size,
                segment.flags
            );
            continue;
        }
        log::debug!(
            "{:?}, {:#x}, {:?}",
            segment.vaddr,
            segment.size,
            segment.flags
        );
        let mut flag = libc::PROT_READ;
        if segment.flags.contains(MappingFlags::EXECUTE) {
            flag |= libc::PROT_EXEC;
        }
        if segment.flags.contains(MappingFlags::WRITE) {
            flag |= libc::PROT_WRITE;
        }
        if let Some(data) = segment.data {
            assert!(data.len() <= segment.size);
            let src = data.as_ptr();
            let dst = segment.vaddr.as_usize() as *mut u8;
            let count = data.len();
            unsafe {
                copy_nonoverlapping(src, dst, count);
                if segment.size > count {
                    core::ptr::write_bytes(dst.add(count), 0, segment.size - count);
                }
            }
        } else {
            unsafe { core::ptr::write_bytes(segment.vaddr.as_usize() as *mut u8, 0, segment.size) };
        }

        unsafe {
            if libc::mprotect(segment.vaddr.as_usize() as _, segment.size, flag)
                == libc::MAP_FAILED as _
            {
                log::error!("vdso: mprotect res failed");
                return Err(());
            }
        };
    }

    for relocate_pair in relocate_pairs {
        let src: usize = relocate_pair.src.into();
        let dst: usize = relocate_pair.dst.into();
        let count = relocate_pair.count;
        log::info!(
            "Relocate: src: 0x{:x}, dst: 0x{:x}, count: {}",
            src,
            dst,
            count
        );
        unsafe { core::ptr::copy_nonoverlapping(src.to_ne_bytes().as_ptr(), dst as *mut u8, count) }
    }

    unsafe { libvdso::init_vdso_vtable(elf_base_addr.unwrap() as _) };

    Ok(vdso_map)
}
