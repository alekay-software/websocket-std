// En C++ (main.cpp)
#include "websocket/websocket-std.h"
#include <stdio.h>
#include <unistd.h>
#include <pthread.h>

#define TRUE 1 

SyncClient *client;

// Función que será ejecutada por el hilo
void *handler(void *arg) {
    while (TRUE) {
        if (!syncClientLoop(client)) { 
            printf("Error in ws loop function");
            break; 
        }
    } 

    return NULL;
}

int main() {
    char cadena[100];
    pthread_t hilo;
    client = syncClientNew();
    syncClientInit(client, "localhost", 3000, "/", NULL);

    if (pthread_create(&hilo, NULL, handler, NULL) != 0) {
        fprintf(stderr, "Error al crear el hilo\n");
        return 1;
    }

    // syncClientSend(client, "Hello from C code :)");
    printf("Hello\n");
    

    while (TRUE) {
        printf("Mensaje: ");
        fgets(cadena, sizeof(cadena), stdin);
        syncClientSend(client, cadena);    
    }

    return 0;
}