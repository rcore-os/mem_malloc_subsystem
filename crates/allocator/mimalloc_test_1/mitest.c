#include <stdio.h>
#include <assert.h>
#include "test_mimalloc.h"
/*
#include <mimalloc.h>

void test_heap(void* p_out) {
  mi_heap_t* heap = mi_heap_new();
  void* p1 = mi_heap_malloc(heap,32);
  void* p2 = mi_heap_malloc(heap,48);
  mi_free(p_out);
  mi_heap_destroy(heap);
  //mi_heap_delete(heap); mi_free(p1); mi_free(p2);
}
*/

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

void mi_test_start(CallBackMalloc _cb1,CallBackMallocAligned _cb2,CallBackFree _cb3) {
  cb1 = _cb1;
  cb2 = _cb2;
  cb3 = _cb3;
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

  //mi_collect(true);
  //mi_stats_print(NULL);
  //return 0;
}
