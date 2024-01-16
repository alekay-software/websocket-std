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

void ws_handler(WSSClient_t* client, int* rs_event, void* data) {
    WSEvent_t event = fromRustEvent(rs_event);
    printf("new event %i \n", event.event);
    if (event.event == WSEvent_CONNECT) { 
        printf("Connected\n");
        if (event.value != NULL) {
            char* msg = (char*) event.value;
            for(int i = 0; i < strlen(msg); i++) {
                printf("%c\n", *msg);
                msg++;
            }
            printf("Message received on connected: %s\n", msg);
        }
    } else if (event.event == WSEvent_CLOSE) {
        WSReason_t* ws_reason = (WSReason_t*) event.value;

        switch (ws_reason->reason)
        {
        case WSREASON_SERVER_CLOSED: 
            printf("Server close the connection C: %u\n", ws_reason->status);
            break;
        case WSREASON_CLIENT_CLOSED: 
            printf("Client close the connection C: %u\n", ws_reason->status);
            break;
        default:
            break;
        }
     
        finish = TRUE;
    } else if (event.event == WSEvent_TEXT) {
        const char* message = (char*) event.value;
        printf("TEXT (%zu): %s\n", strlen(message), message);
        // wssclient_send(client, "Hello from C response");
    }

}

// Función que será ejecutada por el hilo
void *handler(void *arg) {

    // for(int i = 0; i < 1000; i++) {
    //     wssclient_send(client, "Hello from C");
    // }

    // printf("Se han enviado los mensajese en 10 segunos se recibiran todas las respuestas\n");
    // sleep(10);

    while (!finish) {
        pthread_mutex_lock(&mutex);
        WSStatus status = wssclient_loop(client);
       
        if (status != WS_Ok) { 
            finish = TRUE;
            switch (status)
            {
                case WS_HandShakeError:
                    printf("Error in HandShake\n");
                    break;
                case WS_ProtocolError:
                    printf("Protocol error\n");
                    break;

                case WS_IOError: 
                    printf("IOError\n");
                    break;

                case WS_ConnectionClose:
                    printf("Connection close error\n");
                    break;
                
                default:
                    printf("Unknow error\n");
                    break; 
                }
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
        printf("Buy more ram\n");
        return 1; 
    }

    if (pthread_create(&hilo, NULL, handler, NULL) != 0) {
        fprintf(stderr, "Error al crear el hilo\n");
        return 1;
    }

    WSStatus status = wssclient_init(client, "localhost", 3000, "/", ws_handler);
    if (status != WS_Ok) { 
        finish = TRUE;
        switch (status)
        {
        case WS_UnreachableHost:
            printf("Unreachable host\n");
            break;
        
        default:
            break;
        }
    }
    // wssclient_send(client, "First msg form C");
    // while (!finish) {
    //     printf("Mensaje: ");
    //     fgets(cadena, sizeof(cadena), stdin);
    //     pthread_mutex_lock(&mutex);
    //     if (strcmp(cadena, "fin\n") == 0) {
    //         finish = TRUE;
    //     } else {
    //         wssclient_send(client, cadena); 
    //     }
    //     pthread_mutex_unlock(&mutex);
    // }

    // sleep(10);
    // finish = TRUE;

    pthread_join(hilo, NULL);
    wssclient_drop(client);

    return 0;
}
