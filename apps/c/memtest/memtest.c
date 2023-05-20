#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

void* mi_malloc(int size){
    return malloc(size);
}
void mi_free(void* addr,int size){
    free(addr);
}
void* mi_malloc_aligned(int size,int align){
    return malloc(size);
    //return aligned_alloc(align,size);
}

void test_large() {
  const size_t N = 1000;

  for (size_t i = 0; i < N; ++i) {
    size_t sz = 1ull << 21;
    char *a = mi_malloc(sz);
    //char* a = mi_mallocn_tp(char,sz);
    for (size_t k = 0; k < sz; k++) { a[k] = 'x'; }
    mi_free(a,sz);
  }
}

void mi_test_start() {
  printf("running mi_test...\n");
  void* p1 = mi_malloc(16);
  void* p2 = mi_malloc(1000000);
  mi_free(p1,16);
  mi_free(p2,1000000);
  p1 = mi_malloc(16);
  p2 = mi_malloc(16);
  mi_free(p1,16);
  mi_free(p2,16);

  //test_heap(mi_malloc(32));

  p1 = mi_malloc_aligned(64, 8);
  p2 = mi_malloc_aligned(160,8);
  mi_free(p2,160);
  mi_free(p1,64);
  test_large();
  printf("mi_test OK!\n");

  //mi_collect(true);
  //mi_stats_print(NULL);
  //return 0;
}

int main()
{
    srand(time(0));
    puts("Running memory tests...");
    uintptr_t *brk = (uintptr_t *)malloc(0);
    printf("top of heap=%p\n", brk);

    int n = 100;
    int i = 0;
    uintptr_t **p = (uintptr_t **)malloc(n * sizeof(uint64_t));
    printf("%d(+8)Byte allocated: p=%p\n", n * sizeof(uint64_t), p, p[1]);
    printf("allocate %d(+8)Byte for %d times:\n", sizeof(uint64_t), n);
    for (i = 0; i < n; i++) {
        p[i] = (uintptr_t *)malloc(sizeof(uint64_t));
        *p[i] = 233;
        printf("allocated addr=%p\n", p[i]);
    }
    for (i = 0; i < n; i++) {
        free(p[i]);
    }
    free(p);

    mi_test_start();
    //test_aligned();

    puts("Memory tests run OK!");
    return 0;
}
