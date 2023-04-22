#### mimalloc内存分配算法

保证线程安全，但无需使用锁，只需原子操作；

用free-list的分页思想，提高内存的连续性；

使用分离适配的思想，每个页维护相同大小的内存块；

O(1)的单次操作，83%以上的内存利用率，在benchmark上相较于tcmalloc和jemalloc快7%~14%



每个页维护3个链表：空闲块链表free、本线程回收链表local_free，其他线程回收链表thread_free

维护local_free而不是直接回收到free里应该是基于效率的考虑（并非每次free都要收集，而是定期通过调用page_collect来一次性收集）；thread_free的设立是为了线程安全

需要分配时，先从free中取，若free为空，则再调用page_collect：先去尝试收集thread_free，其次再收集local_free。这一机制可以确保page_collect会被定期调用到，可以同时保证效率和不产生浪费的内存块

```cpp
void* _mi_page_malloc(mi_heap_t* heap, mi_page_t* page, size_t size) {
    //从一页中分配内存块的大致逻辑
  mi_block_t* const block = page->free;
  if (block == NULL) {//当free链表为空时，进入generic
    return _mi_malloc_generic(heap, size);      // slow path
  }
  // pop from the free list
  //fast path，直接取链表头即可
  page->used++;
  page->free = block->next;
  return block;
}

void* mi_malloc_generic(mi_heap_t* heap, size_t size) {
    size_t idx = size_class(size);      // 计算得到 size class
    mi_page_queue_t* queue = heap->pages[idx];  // 取相应的page队列
    foreach(page in queue) { // 注：这里还需要限制枚举页的数量，以确保单次调用的时间开销有上限
        page_collect(page);     // 收集页里的可用内存
        if (page->used == 0) {  // 整个页都空闲，回收掉
            page_free(page);
        } else if (page->free != NULL) {    // 收集完如果有可用内存，则重回分配入口
            return mi_heap_malloc(heap, size);
        }
    }
    // 到这儿表明找不到可用的page，从segment分配一个新鲜的page
    ...
}

void page_collect(mi_page_t* page) { // 收集页里的可用内存
  // collect the thread free list
  if (mi_page_thread_free(page) != NULL) {  // quick test to avoid an atomic operation
    _mi_page_thread_free_collect(page);//先收集thread_free
  }
  // and the local free list
  if (page->local_free != NULL) {//如果上一步没收集到东西，再收集local_free
    if (page->free == NULL) {
      // usual case
      page->free = page->local_free;
      page->local_free = NULL;
    }
  }
}
```



每个线程维护线程本地数据结构（段、堆、页），alloc时每个线程只能从自己的堆中分配空间，但是free时任何线程都可以free

因此这里需要保证线程安全，为了避免加锁，采用thread_free链表：其他线程释放本线程的块时，通过原子操作插入到这个块的thread_free里：

```cpp
void atomic_push( block_t** list, block_t* block ) {
  do { block->next = *list; }
  while (!atomic_compare_and_swap(list, block, block->next));
}

atomic_push( &page->thread_free, p );
```



整个数据结构的组织架构如下：

![image-20230423000827253](C:\Users\liuzhangfeiabc\AppData\Roaming\Typora\typora-user-images\image-20230423000827253.png)

每个segment以及pages链表的结构如下：

<img src="C:\Users\liuzhangfeiabc\AppData\Roaming\Typora\typora-user-images\image-20230423004710403.png" alt="image-20230423004710403" style="zoom:67%;" />

每个page的结构如下：

<img src="C:\Users\liuzhangfeiabc\AppData\Roaming\Typora\typora-user-images\image-20230423004755058.png" alt="image-20230423004755058" style="zoom:67%;" />

每个线程维护一个heap：其中的pages字段是一个数组，是根据内存块size大小维护的若干个page的队列（链表），不超过1024的内存块又被pages_direct指针指向，以快速获取（找一个page时，查pages_direct表相比查pages表更快）。

每个segment的开头是一段元数据，pages字段存储的是其中的page的元数据，包括线程号、块大小、free链表、local链表、thread链表等信息；segment的剩余部分就是每个page的地址空间，对于空闲的块，用next指针指向它在链表中的下一个块。



alloc的逻辑如下：

```cpp
void* mi_malloc( size_t n ) {
    heap_t* heap = mi_get_default_heap();   // 取线程相关的堆
    return mi_heap_malloc(heap, size)
}

void* mi_heap_malloc(mi_heap_t* heap, size_t size) {
    if (size <= MI_SMALL_SIZE_MAX) {    // 如果<=1024，进入小对象分配
        return mi_heap_malloc_small(heap, size);
    } else {    // 否则进行通用分配
        return mi_malloc_generic(heap, size)
    }
}

void* mi_heap_malloc_small(mi_heap_t* heap, size_t size) {
    page_t* page = heap->pages_direct[(size+7)>>3]; // 从pages_direct快速得到可分配的页
    block_t* block = page->free;
    if (block != NULL) {                // fast path
        page->free = block->next;
        page->used++;
        return block;
    } else {
        return mi_malloc_generic(heap, size); // slow path
    }
}
```

调用mi_malloc_generic是一个比较慢的过程，程序会在这一阶段进行一些回收工作。程序机制确保它会被每间隔一段时间调用，使得操作的复杂度可以被均摊。



free的逻辑如下，注意如何通过地址找到内存块所在的段和页，以及对于本线程和其他线程的不同处理：

```cpp
void mi_free(void* p) {
    segment_t* segment = (segment_t*)((uintptr_t)p & ~(4*MB));  // 找到对应的segment
    if (segment==NULL) return;
    // 找到对应的page，这是简化过的，第1个page要特殊处理。
    // 因为segment等分成N个page，这里只需要取相对地址，然后除去page的大小，即得到page的索引。
    page_t* page = &segment->pages[(p - segment) >> segment->page_shift];
    block_t* block = (block_t*)p;

    if (thread_id() == segment->thread_id) { // 相同线程，释放到local_free
        block->next = page->local_free;
        page->local_free = block;
        page->used--;
        if (page->used == 0) page_free(page);
    }
    else { // 不同线程，释放到 thread_free
        atomic_push(&page->thread_free, block);
    }
}
```



heap的full存储的是当前已满的页，这是为了保证分配时的效率，避免每次分配时都要遍历所有已满的页面（否则会带来最大30%的效率损失）。

```
This anecdote shows that there is no silver bullet, and an industrial strength memory allocator needs to address many corner cases that might show up only for particular workloads.
```

当一个也从不满变成满时，将其从其所在的pages队列中取出，放进full队列

当一个已满的页被重新释放时，要将其改回不满的状态，但由于释放可能是其他线程引起的，要考虑线程安全问题（要在线程安全的前提下通知原线程这个块已经不再满了），用thread_delayed机制来解决。

一个页用2位二进制来表示状态：当一个页不满时，状态为normal，其他线程的free就将内存块正常插入到这个页的thread_free链表里；但如果已满，状态为delayed，其他线程的free就要插入到原线程的heap的thread_delayed_free链表里，并将状态改为delaying；delaying状态的page在其他线程的free时仍然插入到这个页的thread_free链表里，以保证thread_delayed_free链表里不存在来自相同page的内存块（这同样是为了确保效率，否则会带来最大30%的效率损失）。

等原线程的mi_malloc_generic时，就会遍历这个thread_delayed_free链表，把其中的内存块所在的page重新设为normal状态，并将其从full队列移回到原来的pages队列。

更多的安全措施：如初始建立free链表时以随机顺序连接等，经测试这仅会使程序的性能相较无安全措施的版本慢3%左右。



reference：

[https://www.microsoft.com/en-us/research/uploads/prod/2019/06/mimalloc-tr-v1](https://www.microsoft.com/en-us/research/uploads/prod/2019/06/mimalloc-tr-v1.pdf)[.pdf](https://www.microsoft.com/en-us/research/uploads/prod/2019/06/mimalloc-tr-v1.pdf)

[https://zhuanlan.zhihu.com/](https://zhuanlan.zhihu.com/p/370239503)[p](https://zhuanlan.zhihu.com/p/370239503)[/](https://zhuanlan.zhihu.com/p/370239503)[370239503](https://zhuanlan.zhihu.com/p/370239503)

[https://github.com/microsoft](https://github.com/microsoft/mimalloc)[/mimalloc](https://github.com/microsoft/mimalloc)

[https://github.com/daanx](https://github.com/daanx/mimalloc-bench)[/mimalloc-bench](https://github.com/daanx/mimalloc-bench)