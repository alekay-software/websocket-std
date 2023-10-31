#ifndef _WEBSOCKET_STD_H
#define _WEBSOCKET_STD_H
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum { 
    ON_CONNECT,
    ON_TEXT,
    ON_CLOSE,
} WSEvent_t;

typedef enum {
    con,
    text,
    clos,
} RustEvent_t;

typedef struct _SyncWSClient SyncWSClient;

/*
* Creates a new SyncWSClient or NULL if an error occurred
*/
SyncWSClient *ws_sync_client_new(void);


/*
* Init the websocket, connecting to the given host
* 
* Parameters:
* - SyncWSClient* client
* - const char* host: Server host
* - uint16_t port: Server port
* - const char* path: Server path 
* - void* callback: Callback to execute when an events comes 
*
*
*/
void ws_sync_client_init(SyncWSClient *client,
                    const char *host,
                    uint16_t port,
                    const char *path,
                    void *callback);
                

/*
* Function to execute the internal event loop of the websocket 
* 
* Parameters:
* - SyncWSClient* client
*
* Return:
* The error if the websocket got it.
*
*/
int ws_sync_client_loop(SyncWSClient* client);


/*
* Add a new event in the websocket to send the given message (Text)
* 
* Parameters:
* - SyncWSClient* client
* - message: string to send
*
* Return:
* The error if the websocket got it.
*
*/
int ws_sync_client_send(SyncWSClient* client, const char* message);


/*
* Drop the websocket from memory and close the connection with the server (graceful shutdown)
* 
* Parameters:
* - SyncWSClient* client
*
* Return:
* NULL
*/
void* ws_sync_client_drop(SyncWSClient* client);

WSEvent_t fromRustEvent(RustEvent_t* event);

#endif
