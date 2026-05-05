OUTPUT_ARCH({output_arch})

PHDRS
{
    text_seg  PT_LOAD FILEHDR PHDRS FLAGS(5);   /* RX */
    ro_seg    PT_LOAD FLAGS(4);   /* R  */
    data_seg  PT_LOAD FLAGS(6);   /* RW */
    dynamic   PT_DYNAMIC;
    eh_frame  PT_GNU_EH_FRAME;
    stack     PT_GNU_STACK FLAGS(6);   /* RW, 栈不可执行 */
}

SECTIONS {
    . = SIZEOF_HEADERS;

    /* 动态符号相关节，只读，放在 text_seg */
    .hash           : { *(.hash) } :text_seg
    .gnu.hash       : { *(.gnu.hash) } :text_seg
    .dynsym         : { *(.dynsym) } :text_seg
    .dynstr         : { *(.dynstr) } :text_seg
    .gnu.version    : { *(.gnu.version) } :text_seg
    .gnu.version_d  : { *(.gnu.version_d) } :text_seg
    .gnu.version_r  : { *(.gnu.version_r) } :text_seg

    . = ALIGN(16);
    .text           : {
        *(.text.start)
        *(.text .text.*)
    } :text_seg

    . = ALIGN(4K);
    .plt : { *(.plt .plt.*) } :text_seg

    .pad_text : { BYTE(0); } :text_seg

    /* 只读数据段 */
    . = ALIGN(4K);
    .rodata         : {
        *(.rodata .rodata.* .gnu.linkonce.r.*)
        *(.note.*)
    } :ro_seg

    .eh_frame_hdr   : { *(.eh_frame_hdr) } :ro_seg :eh_frame
    .eh_frame       : { KEEP (*(.eh_frame)) } :ro_seg

    /* 动态段放入 ro_seg，避免和 rw 段共享同一批页 */
    . = ALIGN(4K);
    .dynamic        : { *(.dynamic) } :ro_seg :dynamic

    .pad_ro : { BYTE(0); } :ro_seg

    . = ALIGN(4K);
    .data           : {
        *(.data .data.* .gnu.linkonce.d.*)
        *(.got.plt) *(.got)
    } :data_seg

    . = ALIGN(4K);
    .bss            : {
        *(.bss .bss.* .gnu.linkonce.b.*)
        *(COMMON)
    } :data_seg

    .pad_data : { BYTE(0); } :data_seg
}