#include <stdio.h>
#include <pthread.h>

int count;

void worker(void *arg) {
    for (int i = 0; i < 1000; i++) {
    //    int aux = count; 
    //    count = aux + 1; 
        count++;
    }
}

int main() {
    count = 0;
    pthread_t w1, w2;

    pthread_create(&w1, NULL, worker, NULL);
    pthread_create(&w2, NULL, worker, NULL);

    pthread_join(w1, NULL);
    pthread_join(w2, NULL);

    printf("Counter: %d\n", count);
}