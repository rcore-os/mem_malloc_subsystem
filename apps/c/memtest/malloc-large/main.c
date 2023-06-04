// Test allocation large blocks between 5 and 25 MiB with up to 20 live at any time.
// Provided by Leonid Stolyarov in issue #447 and modified by Daan Leijen.
#include <stdio.h>
#include <stdlib.h>
int get_random(int xmin,int xmax){
  return rand() % (xmax - xmin + 1) + xmin;
}
int main() {
  printf("malloc-large test begin...\n");
  //qemu里的内存只有不到128MB，但实际上超过64MB就容易GG

  static const int kNumBuffers = 8;
  static const size_t kMinBufferSize = 2 * 1024 * 1024;//2MB
  static const size_t kMaxBufferSize = 5 * 1024 * 1024;//5MB
  char* buffers[kNumBuffers];

  //std::random_device rd;
  //std::mt19937 gen(42); //rd());
  //std::uniform_int_distribution<> size_distribution(kMinBufferSize, kMaxBufferSize);
  //std::uniform_int_distribution<> buf_number_distribution(0, kNumBuffers - 1);
  srand(42);
  static const int kNumIterations = 100000;
  //const auto start = std::chrono::steady_clock::now();

  for (int i = 0; i < kNumBuffers; ++i){
    buffers[i] = malloc(kMinBufferSize);
  }

  //printf("*****\n");
  for (int i = 0; i < kNumIterations; ++i) {
    int buffer_idx = get_random(0, kNumBuffers - 1);
    size_t new_size = get_random(kMinBufferSize, kMaxBufferSize);
    //printf("%d %d %d\n",i,buffer_idx,new_size);
    free(buffers[buffer_idx]);
    //printf("*****\n");
    buffers[buffer_idx] = malloc(new_size);
    //printf("%x\n",*(size_t*)(&buffers[buffer_idx]));
  }
  //printf("*****\n");
  for(int i = 0;i < kNumBuffers;++i){
    free(buffers[i]);
  }

  printf("malloc-large test end!\n");
  //const auto end = std::chrono::steady_clock::now();
  //const auto num_ms = std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
  //const auto us_per_allocation = std::chrono::duration_cast<std::chrono::microseconds>(end - start).count() / kNumIterations;
  //std::cout << kNumIterations << " allocations Done in " << num_ms << "ms." << std::endl;
  //std::cout << "Avg " << us_per_allocation << " us per allocation" << std::endl;
  return 0;
}
