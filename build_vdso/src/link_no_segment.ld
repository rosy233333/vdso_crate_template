OUTPUT_ARCH({output_arch})

SECTIONS {
    . = SIZEOF_HEADERS;

    /* 动态符号相关节，只读，放在 text_seg */
    .hash           : { *(.hash) }
    .gnu.hash       : { *(.gnu.hash) }
    .dynsym         : { *(.dynsym) }
    .dynstr         : { *(.dynstr) }
    .gnu.version    : { *(.gnu.version) }
    .gnu.version_d  : { *(.gnu.version_d) }
    .gnu.version_r  : { *(.gnu.version_r) }

    . = ALIGN(16);
    .text           : {
        *(.text.start)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .plt : { *(.plt .plt.*) }

    .pad_text : { BYTE(0); }

    /* 只读数据段 */
    . = ALIGN(4K);
    .rodata         : {
        *(.rodata .rodata.* .gnu.linkonce.r.*)
        *(.note.*)
    }

    .eh_frame_hdr   : { *(.eh_frame_hdr) }
    .eh_frame       : { KEEP (*(.eh_frame)) }

    /* 动态段放入 ro_seg，避免和 rw 段共享同一批页 */
    . = ALIGN(4K);
    .dynamic        : { *(.dynamic) }

    .pad_ro : { BYTE(0); }

    . = ALIGN(4K);
    .data           : {
        *(.data .data.* .gnu.linkonce.d.*)
        *(.got.plt) *(.got)
    }

    . = ALIGN(4K);
    .bss            : {
        *(.bss .bss.* .gnu.linkonce.b.*)
        *(COMMON)
    }

    .pad_data : { BYTE(0); }
}