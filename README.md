# vDSO共享库构建工具

## vDSO简介

**vDSO**（Virtual Dynamic Shared Object，虚拟动态共享对象）是 Linux 内核提供的一种机制，用来高效地将某些内核服务暴露给用户空间，而无需进行用户态到内核态的系统调用（syscall）切换。

其原理为：在内核空间中加载一个**二进制共享库文件（`vDSO`）**，以及这个共享库文件访问的**数据区（`vVAR`）**。之后，将`vDSO`和`vVAR`均映射到用户空间，使用户程序可直接访问`vDSO`中提供的接口，并间接访问`vVAR`的数据。通过该方式，即可实现代码和数据在用户态和内核态间、在不同用户程序间的共享。

为何需要单独的`vVAR`数据区，而不是使用`vDSO`中的数据段？

- 在Linux内核的实现中，`vDSO`被整体映射到一个只读区域。因此`vDSO`的数据段只能存储只读数据，而可变数据需要存储在`vVAR`中。
- 在我们扩展`vDSO`功能的实现中，分别加载`vDSO`的代码段和数据段，并赋予相应的读写权限。但有两种可变数据需要区分：(1)共享的可变数据，在一个地址空间中的修改可以反映到其它地址空间中；(2)私有的可变数据，每个地址空间持有不同的拷贝，修改互不影响。因此我们使用`vVAR`存储共享的可变数据，而使用`vDSO`中的数据段存储私有的可变数据。

关于vDSO的更详细介绍参考[RISC-V Syscall 系列 4：vDSO 实现原理分析](https://tinylab.org/riscv-syscall-part4-vdso-implementation/)。

## 本项目内容

前文已提及，vDSO技术可用于用户和内核之间、用户进程之间的代码与数据共享。其可应用于一些需要如上的代码数据共享的领域，例如共享任务调度、异步系统调用和进程间通信。本项目即使用vDSO机制实现上述的功能。

本仓库中的两个crate，`vdso_helper`和`build_vdso`，用于支持使用Rust语言方便地开发和构建`vDSO`共享库：

- `vdso_helper`：封装了`vDSO`代码所需的操作，包括(1)访问`vVAR`数据和(2)定义可由环境变量修改的常量。
- `build_vdso`：将`vDSO`代码的构建流程整合到`vDSO`外部代码的构建流程中，并构建可被`vDSO`外部代码依赖的API库。

通过它们实现的，`vDSO`开发和构建流程如图所示：

![vdso项目结构](./doc/assets/vdso模板项目结构3.0.png)

基于本项目开展了如下的工作：

- [`vsched`](https://github.com/rosy233333/vipc)：基于vDSO机制的调度器，后续将开发为用户态、内核态的统一调度器。
- [`vqueue`](https://github.com/rosy233333/vqueue)：基于vDSO机制的IPC队列，由[`vipc`](https://github.com/rosy233333/vipc)使用。

目前本项目的工作在[AsyncOS](https://github.com/rosy233333/async-os/tree/vdso-test)上运行，未适配Linux。

## vDSO共享库开发和使用流程

### 开发`vDSO`库

1. 创建`no_std`、`lib`类型的Rust crate，并创建一个模块`api`和源文件`api.rs`。
2. 依赖`vdso_helper`，使用其中的`vvar_data!`和`get_vvar_data!`定义和访问共享数据。
3. 通过声明静态变量的方式声明私有数据。
4. （可选）通过`vdso_helper`中的`mut_cfg!`和`use_mut_cfg!`定义在编译期由环境变量指定的常量。
5. 将暴露出的接口函数放置在`api.rs`中，且使用以下的函数定义：

```Rust
#[unsafe(no_mangle)]
pub extern "C" fn 函数名(参数) -> 返回值 {
    函数体
}
```

注意：不是所有对外提供的函数都需要放入`api.rs`（例如对外提供某些类型关联的方法）。但是，如果该对外提供函数（直接或间接地）访问了共享数据，则必须放入`api.rs`中。

### 构建和使用`vDSO`库

1. 在`vDSO`外部代码的`build.rs`中使用`build_vdso`，配置`BuildConfig`构建参数，并传入`build_vdso`函数以构建`vDSO`库。
2. 执行一次构建后，可在输出目录中找到so文件与API库。
3. 加载`vDSO`和`vVAR`：在外部代码所在的地址空间中映射一块区域，并如此设置：首先保留一块`VvarData`大小的区域，设置为可读可写。在其之后的下一页加载第2步中的so文件，并为各个段设置合适的可读/可写/可执行权限。`vVAR`区域与`vDSO`区域的基址都需要对齐到`config::PAGES_SIZE_4K`。
4. 依赖API库，并传入`vDSO`的加载基址。
5. 通过API库，调用`vDSO`的API。
6. 创建用户进程时，将`vDSO`和`vVAR`映射到其地址空间，并向用户进程传递`vDSO`的基址。用户进程即可通过第4、5步的方式使用vDSO。