#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>

//线程要运行的函数，除了函数名myfunc，其他全都是固定的。
void* myfunc()
{
	printf("Hello World!\n");
	return NULL;
}

int main()
{
	pthread_t th;//在创建线程之前要先定义线程标识符th，相当于int a这样

	pthread_create(&th,NULL,myfunc,NULL);
	/*第一个参数是要创建的线程的地址
	第二个参数是要创建的这个线程的属性，一般为NULL
	第三个参数是这条线程要运行的函数名
	第四个参数三这条线程要运行的函数的参数*/
	
	pthread_join(th,NULL);
	/*线程等待函数，等待子线程都结束之后，整个程序才能结束
	第一个参数是子线程标识符，第二个参数是用户定义的指针用来存储线程结束时的返回值*/
	return 0;
}
