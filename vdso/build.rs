// Copied and modified from https://github.com/AsyncModules/vsched/blob/e19b572714a6931972f1428e42d43cc34bcf47f2/vsched/build.rs
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("linker.lds");
    let arch = option_env!("ARCH").unwrap();
    let arch_lds = match arch {
        "riscv64" => "riscv",
        "aarch64" => "aarch64",
        "x86_64" => "i386:x86-64",
        _ => panic!("Unsupported arch"),
    };
    let linker = format!(
        r#"
OUTPUT_ARCH({})


SECTIONS {{
    . = SIZEOF_HEADERS;

    /* 先放置动态链接相关的只读段 */
    .hash		: {{ *(.hash) }}			:text
	.gnu.hash	: {{ *(.gnu.hash) }}
	.dynsym		: {{ *(.dynsym) }}
	.dynstr		: {{ *(.dynstr) }}
	.gnu.version	: {{ *(.gnu.version) }}
	.gnu.version_d	: {{ *(.gnu.version_d) }}
	.gnu.version_r	: {{ *(.gnu.version_r) }}

    /* 动态段单独分配 */
    .dynamic    : {{ *(.dynamic) }} :dynamic

    . = ALIGN(16);
    /* 代码段（.text）需要放在只读数据段之前 */
    .text       : {{
        *(.text.start)
        *(.text .text.*)
    }} :text

    . = ALIGN(4K);
    /* 只读数据段（.rodata等） */
    .rodata     : {{
        *(.rodata .rodata.* .gnu.linkonce.r.*)
        *(.note.*)
    }} :rodata

    . = ALIGN(4K);
    /* 数据段（.data、.bss等）单独分配 */
    .data       : {{
        *(.data .data.* .gnu.linkonce.d.*)
        *(.got.plt) *(.got)
    }} :data

    . = ALIGN(4K);
    .bss        : {{
        *(.bss .bss.* .gnu.linkonce.b.*)
        *(COMMON)
    }} :bss

    .pad : {{
        __pad_start = .;
        BYTE(0);
        . = ALIGN(4K);
        __pad_end = .;
    }}

    .eh_frame_hdr	: {{ *(.eh_frame_hdr) }}		:text	:eh_frame_hdr
	.eh_frame	: {{ KEEP (*(.eh_frame)) }}	:text
}}

PHDRS {{
    text    PT_LOAD FLAGS(5) FILEHDR PHDRS;  /* 代码段 */
    rodata  PT_LOAD FLAGS(5);                /* 只读数据段 */
    data    PT_LOAD FLAGS(6);                /* 数据段（可读写） */
    bss    PT_LOAD FLAGS(6);                /* 数据段（可读写） */
    dynamic PT_DYNAMIC FLAGS(4);             /* 动态段 */
    note    PT_NOTE FLAGS(4);                /* 注释段 */
    eh_frame_hdr PT_GNU_EH_FRAME;            /* EH帧头 */
}}
"#,
        arch_lds
    );
    std::fs::write(&out_path, linker).unwrap();
    println!("cargo:rustc-link-arg=-T{}", out_path.display());
}
