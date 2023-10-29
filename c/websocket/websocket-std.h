#ifndef _WEBSOCKET_STD_H
#define _WEBSOCKET_STD_H
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

// typedef enum ConnectionStatus {
//   NOT_INIT,
//   HANDSHAKE,
//   OPEN,
//   CLIENT_WANTS_TO_CLOSE,
//   SERVER_WANTS_TO_CLOSE,
//   CLOSE,
// } ConnectionStatus;

// typedef struct Extension {} Extension;

// typedef enum {
//     WEBSOCKET_DATA,
//     HTTP_RESPONSE,
//     HTTP_REQUEST,
//     NO_DATA
// } Event;

// typedef struct {
//     const char* host;
//     uint16_t port;
//     const char* path;
//     ConnectionStatus connection_status;
//     uint64_t message_size;
//     // Podrías usar un tipo adecuado para representar la duración, como un valor en segundos.
//     uint64_t timeout_seconds;
//     void* stream;
//     unsigned char* recv_storage;
//     size_t recv_storage_size;
//     unsigned char* recv_data;
//     size_t recv_data_size;
//     void* cb_data;
//     // Define un tipo de función que represente el callback.
//     void (*callback)(void* client, int eventType, void* data);
//     char* protocol;
//     // En lugar de usar un vector dinámico, puedes utilizar un array estático con un tamaño máximo.
//     Extension extensions [10];
//     // En lugar de usar VecDeque, puedes implementar tus propios mecanismos para trabajar con colas.
//     Event input_events[100];
//     Event output_events[100];
//     char websocket_key[128];
//     size_t close_iters;
// } SyncClient;

typedef struct {} SyncClient;

SyncClient *syncClientNew(void);

void syncClientInit(SyncClient *client,
                    const char *host,
                    uint16_t port,
                    const char *path,
                    void *callback);
                
int syncClientLoop(SyncClient* client);
int syncClientSend(SyncClient* client, const char* message);

#endif