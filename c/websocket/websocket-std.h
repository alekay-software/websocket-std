#ifndef _WEBSOCKET_STD_H
#define _WEBSOCKET_STD_H
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum { 
    WSEvent_CONNECT,
    WSEvent_TEXT,
    WSEvent_CLOSE,
} WSEvent_t;

typedef enum _OpaqueRustEvent_t RustEvent_t;

typedef struct _OpaqueWSSClient_t WSSClient_t;

/*
* Creates a new WSSClient_t or NULL if an error occurred
*/
WSSClient_t *wssclient_new(void);


/*
* Init the websocket, connecting to the given host
* 
* Parameters:
* - WSSClient_t* client
* - const char* host: Server host
* - uint16_t port: Server port
* - const char* path: Server path 
* - void* callback: Callback to execute when an events comes 
*
*
*/
void wssclient_init(WSSClient_t *client,
                    const char *host,
                    uint16_t port,
                    const char *path,
                    void *callback);
                

/*
* Function to execute the internal event loop of the websocket 
* 
* Parameters:
* - WSSClient_t* client
*
* Return:
* The error if the websocket got it.
*
*/
int wssclient_loop(WSSClient_t* client);


/*
* Add a new event in the websocket to send the given message (Text)
* 
* Parameters:
* - WSSClient_t* client
* - message: string to send
*
* Return:
* The error if the websocket got it.
*
*/
int wssclient_send(WSSClient_t* client, const char* message);


/*
* Drop the websocket from memory and close the connection with the server (graceful shutdown)
* 
* Parameters:
* - WSSClient_t* client
*
*/
void wssclient_drop(WSSClient_t* client);

WSEvent_t fromRustEvent(RustEvent_t* event);

#endif
