# vdso模板重构-debug笔记

## 链接与重定位相关bug

### 编译时链接bug

在将声明和使用共享数据的功能分离到`vdso_helper`库后，在编译时出现了链接器错误：

```
section .rela.dyn LMA overlaps section .data LMA
```

查询得知，LMA指加载地址，该错误表明两个段的加载地址发生了重叠。这是由`vdso`模块中的自定义链接脚本导致的问题，因为如果不使用自定义链接脚本，则不会产生该问题。然而，为了保证`get_code_base`函数（原先的`get_data_base`函数）获取到正确的地址，需要使用自定义的链接脚本。

询问deepseek有关overlap问题的几种解决方案后，发现其中一种方式是在链接脚本中显式声明某些段。因为链接脚本中未声明的段会自动分配空间，而这部分空间可能与显式声明的段的空间重叠。（实际上，在上文的报错中，的确是`.rela.dyn`未声明、`.data`有声明）。

根据以上思路，将出现错误的段显式声明出来，解决了该问题。

### 运行时重定位bug

解决了如上问题后，在运行时又出现了错误。在用户态测试中，直接报段错误，没有其它提示。在AsyncOS的内核态测试中，得知了出现`CodePageFault`，以及出错的`pc`。

出错的`pc`的值落在了正常的代码范围之外，且高位全部为0，可以得知原因是某个函数没有重定位，调用该函数时跳转到了其以0x0为基址的加载地址，而非实际的加载地址导致的。

通过`riscv64-unknown-elf-objdump -s -d libvdsoexample.so > disasm.s`反编译so文件，并与用户态测试中的CPU log`qemu.log`、内核态测试中Trapframe中的`ra`比较，得知了出错代码附近的指令执行流程：

代码首先想调用`get_code_base`函数，其先查询`.got.plt`表，再调用动态链接器重定位`get_code_base`，最后跳转到GOT表（在我们的链接脚本中，放在了`.data`段中），获得`get_code_base`的实际地址。然而，由于我们的加载流程中，只在加载时进行了重定位，而没有提供运行时重定位的支持，因此其获得的`get_code_base`的地址仍是定位前的地址，导致了错误。（这一段流程可见[链接、装载与库/动态链接/延迟绑定](https://github.com/rosy233333/weekly-progress/blob/master/25.3.13~25.3.19/%E3%80%8A%E7%A8%8B%E5%BA%8F%E5%91%98%E7%9A%84%E8%87%AA%E6%88%91%E4%BF%AE%E5%85%BB--%E9%93%BE%E6%8E%A5%E3%80%81%E8%A3%85%E8%BD%BD%E4%B8%8E%E5%BA%93%E3%80%8B%E9%98%85%E8%AF%BB%E7%AC%94%E8%AE%B0.md#%E5%BB%B6%E8%BF%9F%E7%BB%91%E5%AE%9Aplt)）

进一步发现，导致了该错误的`get_code_base`的重定位项，类型为`R_RISCV_JUMP_SLOT`。在RISC-V文档和`elf-parser`的实现中，对该重定位项的处理都是保持不动，等到运行时再调用动态链接器进行重定位。因为我们没有运行时重定位的支持，因此我将`elf-parser`对该类重定位项的处理改为了在加载时重定位（也就是，加上了加载基址）。经过这样的处理后，就可以正常运行了。

### 私有数据访问bug

在这之后，遇到了调用`get_private_data`访问私有数据时报段错误的问题。

因为发现私有数据设为0时出现问题，设为1时则没有问题，考虑到是`.bss`段出现了问题。`.bss`段不包含在文件中，导致SO文件的加载大小大于文件大小。但之前是按文件大小进行分配空间，因此`.bss`段超出了范围。增加分配的空间，直接多分配一页，解决了该问题。

在解决该问题的过程中，还发现了已存在的其它问题：

- Async-OS中的加载机制只是拷贝了SO文件，并没有按Segment加载。因此修改了Async-OS的加载机制。
- 使用`include_bytes`加载SO文件时，可能出现不对齐的问题。因此改为了`include_bytes_aligned`库提供的`include_bytes_aligned`宏。

## 编译时的bug

### 在`user_test`上编译遇到的bug

拆分出用于编译vdso编译单元的`build_vdso`模块后，发现其编译函数在单元测试时可以正常运行，而在集成进`user_test`时则会报错。

最终得知了原因：在`build.rs`中使用`Command`调用`cargo`时，其创建的进程的环境变量与主进程的环境变量相同。而`cargo`、`rustc`等程序在运行时会向进程中添加环境变量，这些环境变量就会影响到使用`Command`调用的、用于编译vdso编译单元的`cargo`。

出现问题的是`crt-static`相关的设置，因此通过如下代码阻断了该设置通过环境变量的传递。

```Rust
// 如果启用了crt-static特性，则在vdso的编译中去掉该特性，否则会报错
if let Ok(value) = env::var("CARGO_CFG_TARGET_FEATURE") {
    if value.contains("crt-static") {
        let mut vdso_value = value.replace(",crt-static", "");
        if vdso_value == value {
            vdso_value = value.replace("crt-static,", "")
        }
        if vdso_value == value {
            // 说明该变量只指定了crt-static一项
            cargo.env_remove("CARGO_CFG_TARGET_FEATURE");
        } else {
            cargo.env("CARGO_CFG_TARGET_FEATURE", vdso_value);
        }
    }
}
if let Ok(value) = env::var("CARGO_ENCODED_RUSTFLAGS") {
    if value.contains("+crt-static") {
        let vdso_value = value.replace("+crt-static", "-crt-static");
        cargo.env("CARGO_ENCODED_RUSTFLAGS", vdso_value);
    }
}
```

之后，可以正常运行。

### 在AsyncOS上编译遇到的bug

在AsyncOS上运行该模板时，遇到了“在vdso的编译过程中，找不到`hal`依赖库”的bug。（`hal`库为`vdso_helper`依赖的库。）但如果使用user_test，则在同样的编译流程中没有出现此bug。

分析后发现，是因为使用不同的工具链版本导致的该问题。这意味着，主编译单元使用的工具链版本影响到了vdso编译单元的编译。因此，需要尽可能消除这一影响。

之前已得知，主编译单元添加的环境变量会影响到vdso编译单元。因此，在比对了添加前后的环境变量列表后发现，主编译单元添加的环境变量均带有`"RUST"`或`"CARGO"`字符串。因此，根据该特征清除了所有相应的环境变量，即可为vdso编译单元的`cargo`创建一个干净的执行环境。

如上设置后，仍出现编译错误。依然找不到`hal`依赖库，不过报错的主体从vdso编译单元变更为了主编译单元。分析依赖关系后得知，主编译单元也会依赖`hal`库，因此出现了版本不兼容导致的错误。

首先，认为可能是因为`hal`库没有位于git仓库根目录引起的。因此，将`hal`库从`vsched`中分离出来，分别尝试了git方式和路径方式的依赖，均出现报错。且该报错明确了错误原因就是工具链版本不兼容。

其次，尝试升级AsyncOS使用的工具链版本，但失败。新版的工具链会导致一些汇编宏无法被识别出来。

最后，修改`hal`库，使其与AsyncOS所在的工具链版本（nightly-2024-11-05）兼容：

- 将`Cargo.toml`中的`edition`由`2024`改为`2021`。
- 更改了`hal`的依赖库，`page_table_entry`的版本，降级到AsyncOS使用的旧版。
- 在裸函数（naked function）的语法上，`nightly-2024-11-05`与最新版存在区别：前者需要在开启相应cargo feature后，使用`#[naked]`；后者不需要相应cargo feature，使用`#[unsafe(naked)]`。因为使用裸函数会导致无法同时兼容两个工具链版本，因此将代码中涉及的所有裸函数全都改写为“内联汇编+`extern "C"`”的形式，实现了对两个版本的兼容。

进行这些修改后，vdso共享库可以兼容两个工具链版本，也可以在AsyncOS上正常运行了。
