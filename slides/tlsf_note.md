#### TLSF内存分配算法

支持：O(1)的malloc与dealloc；每次分配的额外空间开销仅为4字节（默认4字节对齐，64位机器则改为8字节）；内存碎片较少；支持动态添加和删除内存池；不保证线程安全，需要调用者保证

Segregated Fit算法：基于一组链表，每个链表包含特定大小范围的空闲块

Two Level：采用两级链表机制

第1层：将块按照2的幂进行分类：如 $[2^4,2^5),[2^5,2^6)$ ……

第1层链表的指针指向一组第2层链表的表头，分别表示将这个区间进一步细分

如 $[2^6,2^7)$ 可进一步细分为 $[2^6,2^6+2^4),[2^6+2^4,2^6+2^5),[2^6+2^5,2^6+3*2^4),[2^6+3*2^4,2^7)$

由一个大小定位到内存块，可以通过先取log2获得第1级位置，再取次高的若干位二进制（上述例子为2位）来实现

<img src="C:\Users\liuzhangfeiabc\AppData\Roaming\Typora\typora-user-images\image-20230421202506388.png" alt="image-20230421202506388" style="zoom:50%;" />

<img src="C:\Users\liuzhangfeiabc\AppData\Roaming\Typora\typora-user-images\image-20230421202525282.png" alt="image-20230421202525282" style="zoom:80%;" />



tlsf内存块头结构：

```cpp
typedef struct block_header_t//tlsf内存块头结构
{
	/* Points to the previous physical block. */
	struct block_header_t* prev_phys_block;//内存地址上上一个块的位置指针
	//只存储上一个块的位置，是因为下一个块可以根据这个块的大小算出来

	/* The size of this block, excluding the block header. */
	size_t size;//这个块的大小，注意是不包括分配时要带的8字节块头大小的
	//因为块大小是4对齐的，所以用低2位分别表示这个块和上一个块是否是free的

	/* Next and previous free blocks. */
	struct block_header_t* next_free;//free链表中的下一个块
	struct block_header_t* prev_free;//free链表中的上一个块
	//free链表只对free状态的块使用
} block_header_t;
```



整个tlsf的控制头结构：

```cpp
typedef struct control_t//整个tlsf的控制结构
{
	/* Empty lists point at this block to indicate they are free. */
	block_header_t block_null;//空块

	/* Bitmaps for free lists. */
	unsigned int fl_bitmap;//一级链表的bitmap，标记每个一级链表是否非空
	unsigned int sl_bitmap[FL_INDEX_COUNT];//二级链表的bitmap，标记每个二级链表是否非空

	/* Head of free lists. */
	block_header_t* blocks[FL_INDEX_COUNT][SL_INDEX_COUNT];//二级链表结构
    //SL_INDEX_COUNT=32表示二级链表将一级链表的一个区间拆分成了32段，也就是要根据最高位后的5个二进制位来判断
} control_t;
```



将大小小于256的块单独维护（位于一级链表的0下标处，也要按照第二级链表维护），其余块按照两级链表结构维护



malloc的策略：

1、先将所需size上取整到下一个二级块大小的下界，并找到相应的一、二级链表；

2、查询是否存在一个符合要求的块：先在相应的一级链表中找：从给定的二级链表开始第一个非空的二级链表，如果存在就从中取一个块；如果不存在，就向上找第一个非空的一级链表，再找其中第一个非空的二级链表；

3、如果分配出去块比所需内存大很多，可以split，要求至少大一个block_header_t的大小（32字节）；

4、如果malloc时额外要求地址对齐（大于8字节）：则要找到一个足够大的块，使得在块上找到对齐的地址后，前面留下的部分拆成一个更小的块（至少32字节，如果不足则将分配出去的地址继续向后移动），之后仍然有足够的空间。



free的策略：物理上连续的空闲内存块要前后合并，然后插入回相应的二级链表中



realloc的策略：先看当前块和物理上的下一块（如果空闲的话）加起来够不够用

如果够用，就直接与下一块合并（如果空闲的话）再在原位重新分配

否则，直接重新alloc然后free掉原来的



reference：

http://www.gii.upv.es/tlsf/files/ecrts04_tlsf.pdf

https://github.com/mattconte/tlsf