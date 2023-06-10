### 组件化OS--aceros的改进：内存分配算法

致理-信计01 张艺缤 zhangyb20@mails.tsinghua.edu.cn



#### 功能实现

- `basic allocator`：路径为 `crates/basic_allocator`
  - 支持 `first fit, best fit, worst fit` 三种分配策略
- `TLSF`：路径为 `crates/TLSF_allocator`
  - 实现了Rust语言和C语言版（通过ffi接入）
- `mimalloc`：路径为 `crates/mimalloc_allocator`
  - Rust语言，目前为单线程版本
- `crates/allocator` 中分别对应实现了上述内存分配器的接入模块
- `modules/axalloc` 中支持通过 `features` 选择不同的内存分配器，默认为 `mimalloc`
  - 可以通过修改 `modules/axalloc/Cargo.toml` 来切换
- 用户态测试：测试用例集成在 `crates/allocator_test` 中，测试框架位于 `crates/allocator/tests`
  - Rust语言测例：`basic, align_test, multi_thread_test`
  - C语言测例：`mitest, glibc_bench, malloc_large`



#### 内存分配概述

内存分配器需要实现的功能：

- init (addr, size)：初始化内存分配器，向内存分配器提供一段[addr,addr+size)的内存空间；

- add_memory (addr, size)：向内存分配器额外增加一段[addr,addr+size)的内存空间；

- alloc (size, align)：申请一段空间，大小为size，地址对齐要求为align（为2的幂，一般为8字节），返回分配空间的起始地址addr；

- dealloc (addr, size, align)：释放一段先前申请的空间，起始地址为addr，大小为size，地址对齐要求为align。

评价内存分配算法的指标：

- 性能：通常要求内存分配器单次O(1)；不仅取决于性能分配器的效率本身，还有分配内存的连续性等各种因素；

- 空间利用率：尽可能高，内部碎块与外部碎块尽可能少；

- 线程安全等。



#### 算法介绍

##### （1）basic allocator

将全局所有的空闲块用一个链表来维护

- Alloc时，查找一个合适的空闲块，切割为合适大小后分配

- Free时，将其与前后的空闲块合并后插入回链表中

- 根据查找空闲块的策略不同，分为first fit、best fit、worst fit三种

  - first fit：选取第一个大小足够的内存块
  - best fit：选取大小足够的内存块中最小的

  - worst fit：选取大小足够的内存块中最大的

优点：实现相对容易；消除了内碎块

缺点：可能存在外碎块；查找空闲块时最坏需要遍历整个链表，效率低

上述代码约600行，于第8周完成。



##### （2）TLSF

用两级链表来维护大小在一定范围内的内存块

O(1)的malloc与dealloc；

每次分配的额外空间开销仅为8字节；

内存碎片较少；

支持动态添加和删除内存池；

详细介绍见文档 `tlsf_note.md`。

优点：单次操作复杂度严格O(1)；内碎块相对Buddy和slab更少

- 取决于二级链表的大小，如取5位则内碎块不超过1/64

缺点：

- 每次操作时，拆分和合并内存块的开销相对较大；

- 多次申请空间很可能不连续；

- 最小分配单位为16字节，分配小内存块时冗余较大

分别使用Rust语言实现和接入C语言的既有实现，Rust代码量约1000行，于第11周完成。



##### （3）mimalloc

原始算法是保证线程安全的：即可以支持多线程同时申请/释放，无需上锁（Mutex），仅需原子操作（Atomic）

但算法相对较复杂（原版C代码~3500行），目前实现的Rust版本为单线程的简化版

在mimalloc内存分配器中，内存维护单元分为：堆（Heap）、段（Segment）、页（Page）、块（Block）

每个Page中的块大小都是相同的

Segment以4MB对齐，承载各种Page

每个线程用一个Heap作为mimalloc的控制结构，核心为维护不同block大小的Page链表

详细介绍见文档 `mimalloc_note.md`。

优点：单次操作O(1)；连续分配时内存地址大致连续；速度较现在通用的内存分配器快约7%~14%；内碎块相对较小；

缺点：

- 以段（4MB）和页（64KB）为单位维护内存，在分配请求不规则时可能有较大冗余；

- 算法较为复杂。

当前的Rust语言单线程版本，代码量约1000行，于第15周完成。



#### APP测试

主要用于测试内存分配的两个app为 `apps/memtest` 和 `apps/c/memtest`



#### 用户态测试

在 `crates/allocator/tests` 中搭建了专用于用户态测试的global_allocator和测试框架

- global_allocator接入了上述各种内存分配器，以及rust自带的System分配器（linux中，使用的是__libc_malloc）

- 通过init_heap函数接入一个全局static的大数组（512MB）用于内存分配

- 通过各种分配器对应的init函数来动态切换使用的分配器

- 实现了各种测试用例的接口以及各种分配器的测试入口，集成为一个test

- 多线程支持：在每个内存分配器外面套一层mutex

新建了`crates/allocator_test`，内部集成了各种测试用例，可以在运行用户态测试时调用

各个测试用例的介绍：

##### (1) Basic

- Rust语言

- 与apps/memtest类似

- 主要为大量Vec和btreemap测试

##### (2) Align_test

- Rust语言，自行编写

- 测试align为非8字节的情况

- 仅有System、TLSF(C和Rust)、mimalloc支持该功能

##### (3) Multi_thread_test

- Rust语言，自行编写

- 测试多线程情况下的内存分配效率

##### (4) Mitest

- C语言

- Mimalloc仓库中自带的测试用例

- 仓库地址：https://github.com/microsoft/mimalloc

- 主要测试算法的正确性

##### (5) Glibc_bench

- C语言

- 仓库地址：https://github.com/daanx/mimalloc-bench

- 位于`bench/glibc-bench`

- 一系列比较复杂的内存操作，测试各种算法的性能

##### (6) Malloc_large

- C语言

- 仓库地址：https://github.com/daanx/mimalloc-bench

- 位于`bench/malloc-large`

- 用于测试对大内存（2~15MB）的申请与释放

##### (7) Multi_thread_c_test

- C语言，自行编写

- 功能大致类似于multi_thread_test

- 测试C语言下的多线程内存分配



#### 运行用户态测试

```
cargo test -p allocator --release -- --nocapture
```



#### 运行示例app

```
make A=apps/memtest ARCH=riscv64 LOG=info run
make A=apps/c/memtest ARCH=riscv64 LOG=info run
```



#### 用户态测试结果

注：单位均为秒（s）；“——”表示未实现相关功能

| 算法           | system | buddy | slab  | first fit | best fit | worst fit | TLSF C | TLSF Rust | mimalloc |
| -------------- | ------ | ----- | ----- | --------- | -------- | --------- | ------ | --------- | -------- |
| basic          | 0.361  | 8.251 | 0.584 | 0.346     | 1.79     | 14.557    | 0.346  | 0.35      | 0.336    |
| align test     | 0.019  | 0     | 0     | 0         | 0        | 0         | 0.011  | 0.009     | 0.008    |
| multi thread   | 3.223  | 6.11  | 3.275 | 3.25      | 3.524    | 8.965     | 3.208  | 3.206     | 3.225    |
| mitest         | 0.054  | 0.051 | 0.052 | 0.072     | 0.069    | 0.069     | 0.054  | 0.069     | 0.052    |
| glibc bench    | 1.539  | 1.727 | 1.39  | 2.086     | 2.37     | 2.382     | 2.374  | 3.028     | 2.198    |
| malloc large   | 0.088  | 0.007 | 0.006 | 0.03      | 0.024    | 0.201     | 0.021  | 0.022     | 0.01     |
| multi thread C | 0.444  | 5.429 | 0.53  | 0.621     | 1.374    | 8.174     | 0.651  | 0.778     | 0.521    |



#### 测试结果图表

##### basic：

![basic](pic\basic.png)



##### align test：

![align](pic\align.png)



##### multi thread：

![multi_thread](pic\multi_thread.png)



##### mitest：

![mitest](pic\mitest.png)



##### glibc bench：

![glibc_bench](pic\glibc_bench.png)



##### malloc large：

![malloc_large](pic\malloc_large.png)



##### multi thread C：

![multi_thread_c](pic\multi_thread_c.png)



#### TODO

（1）完成Rust版本mimalloc的多线程支持；

（2）将C语言的mimalloc实现接入arceos   仓库链接：https://github.com/microsoft/mimalloc

（3）增加更多多线程测例，以测试mimalloc对多线程的支持   仓库链接：https://github.com/daanx/mimalloc-bench

