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



#### 用户态测试

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

