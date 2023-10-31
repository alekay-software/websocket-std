// En C++ (main.cpp)
#include "websocket/websocket-std.h"
#include <stdio.h>
#include <unistd.h>
#include <pthread.h>
#include <string.h>
#include <stdlib.h>

#define FALSE 0
#define TRUE 1 

WSSClient_t *client;
pthread_mutex_t mutex;
int finish;

void ws_handler(WSSClient_t* client, RustEvent_t* rs_event, void* data) {
    WSEvent_t event = fromRustEvent(rs_event);
    printf("new event %i \n", event);
    if (event == WSEvent_CONNECT) { 
        printf("Connected\n");
    } else if (event == WSEvent_CLOSE) {
        printf("Closed\n");
    } else if (event == WSEvent_TEXT) {
        printf("TEXT\n");
    }

}

// Función que será ejecutada por el hilo
void *handler(void *arg) {
    while (!finish) {
        pthread_mutex_lock(&mutex);
        if (!wssclient_loop(client)) { 
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
    
    client = wssclient_new();
    if ( client == NULL ) {
      printf("Client is NULL");
    }

    if (pthread_create(&hilo, NULL, handler, NULL) != 0) {
        fprintf(stderr, "Error al crear el hilo\n");
        return 1;
    }

    printf("Hello\n");
    wssclient_init(client, "localhost", 3000, "/", ws_handler);
    printf("hello 2\n");

    while (!finish) {
        printf("Mensaje: ");
        fgets(cadena, sizeof(cadena), stdin);
        pthread_mutex_lock(&mutex);
        if (strcmp(cadena, "fin\n") == 0) {
            finish = TRUE;
        } else {
            wssclient_send(client, cadena); 
        }
        pthread_mutex_unlock(&mutex);
    }

    pthread_join(hilo, NULL);
    wssclient_drop(client);

    wssclient_loop(client);
    wssclient_loop(client);
               
    if (client == NULL) {
        printf("Is null\n");
    } else {
        printf("Client is not null");
    }

    return 0;
}
