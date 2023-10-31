// En C++ (main.cpp)
#include "websocket/websocket-std.h"
#include <stdio.h>
#include <unistd.h>
#include <pthread.h>
#include <string.h>
#include <stdlib.h>

#define FALSE 0
#define TRUE 1 

SyncWSClient *client;
pthread_mutex_t mutex;
int finish;

void ws_handler(SyncWSClient* client, RustEvent_t* rustEvent, void* data) {
    WSEvent_t event = fromRustEvent(rustEvent);
    printf("new event %i \n", event);
   
    if (event == ON_CONNECT) { 
        printf("Connected\n");
    } else if (event == ON_CLOSE) {
        printf("Closed\n");
    } else if (event == ON_TEXT) {
        printf("TEXT\n");
    }
}

// Función que será ejecutada por el hilo
void *handler(void *arg) {
    while (!finish) {
        pthread_mutex_lock(&mutex);
        if (!SyncWSClientLoop(client)) { 
            printf("Error in ws loop function");
            break; 
        }
        pthread_mutex_unlock(&mutex);
    } 

    return NULL;
}

int main() {
    char cadena[100];
    pthread_t hilo;
    finish = FALSE;
    pthread_mutex_init(&mutex, NULL);
    
    client = SyncWSClientNew();

    if (pthread_create(&hilo, NULL, handler, NULL) != 0) {
        fprintf(stderr, "Error al crear el hilo\n");
        return 1;
    }

    SyncWSClientInit(client, "localhost", 3000, "/", ws_handler);
    // SyncWSClientSend(client, "Hello from C code :)");

    while (!finish) {
        printf("Mensaje: ");
        fgets(cadena, sizeof(cadena), stdin);
        pthread_mutex_lock(&mutex);
        if (strcmp(cadena, "fin\n") == 0) {
            finish = TRUE;
        } else {
            SyncWSClientSend(client, cadena); 
        }
        pthread_mutex_unlock(&mutex);
    }

    pthread_join(hilo, NULL);
    SyncWSClientDrop(client);

    // int x = SyncWSClientLoop(client);
    // printf("Error %i\n, ", x);
    SyncWSClientLoop(client);
    SyncWSClientLoop(client);
    // SyncWSClientLoop(client);
               
    if (client == NULL) {
        printf("Is null\n");
    } else {
        printf("Client is not null");
    }

    return 0;
}