#include <websocket-std.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <pthread.h>

#define FALSE 0
#define TRUE 1 

int total = 0;

void ws_handler(WSSClient_t* client, RustEvent rs_event, void* data) {
    // This function is required because the rust events are not compatible with C.
    // It will return a WSEvent_t struct compatible with C.
    WSEvent_t event = from_rust_event(rs_event);
    if (event.kind == WSEvent_CONNECT) { 
        if (event.value != NULL) {
            char* msg = (char*) event.value;
            printf("Message received on connected: %s\n", msg);
        }
        wssclient_send(client, "Connection complete");
    } else if (event.kind == WSEvent_CLOSE) {
        WSReason_t* ws_reason = (WSReason_t*) event.value;

        switch (ws_reason->reason) {
            case WSREASON_SERVER_CLOSED: 
                printf("Server close the connection C: %u\n", ws_reason->status);
                break;
            case WSREASON_CLIENT_CLOSED: 
                printf("Client close the connection C: %u\n", ws_reason->status);
                break;
            default:
                break;
        }
    } else if (event.kind == WSEvent_TEXT) {
        total++; 
        const char* message = (char*) event.value;
        printf("TEXT (%zu): %s\n", strlen(message), message);
        wssclient_send(client, "Hello from C response");
    }

}

void *handler(void *arg) {
    WSSClient_t *client = (WSSClient_t*) arg;

    time_t start, end;
    time(&start);

    while (TRUE) {
        time(&end);
        if (difftime(end, start) >= 10) { break; }
        WSStatus status = wssclient_loop(client);
       
        if (status != WSStatusOK) { 
            switch (status)
            {
                case WSStatusHandShakeError:
                    printf("Error in HandShake\n");
                    break;
                case WSStatusUnreachableHost:
                    printf("UnreachableHost\n");
                    break;

                case WSStatusIOError: 
                    printf("IOError\n");
                    break;

                case WSStatusConnectionCloseError:
                    printf("Connection close error\n");
                    break;
                
                case WSStatusDecodingFromUTF8Error:
                    printf("Error decoding frame from utf8\n");
                    break;

                case WSStatusInvalidFrame:
                    printf("Invalid frame received\n");
                    break;
                
                default:
                    printf("Unknow error\n");
                    break; 
                }
            break;
        }
    } 
    return NULL;
}

int main() {
    char cadena[100];
    WSSClient_t *client;
    pthread_t thread;

    // Create new websocket
    client = wssclient_new();

    if (client == NULL) {
        printf("Buy more ram\n");
        return 1; 
    }

    // The websocket will be managed by a thread but this is not necessary, just to show
    // that the websocket is capable os do that.
    if (pthread_create(&thread, NULL, handler, client) != 0) {
        fprintf(stderr, "Error al crear el hilo\n");
        return 1;
    }

    // Init connection
    wssclient_init(client, "localhost", 3000, "/", ws_handler);

    pthread_join(thread, NULL);

    // Clean the memory used by the websocket and close the connection gracefully
    wssclient_drop(client);

    printf("Total messages received: %d: \n", total);

    return 0;
}
