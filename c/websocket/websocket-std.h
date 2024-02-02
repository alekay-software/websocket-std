#ifndef _WEBSOCKET_STD_H
#define _WEBSOCKET_STD_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

// TODO: Change the name of the enum that doesn't contain the error word (to not be confused if this value is an error or not)
typedef enum { 
    WSStatusOK,
    WSStatusUnreachableHost,
    WSStatusHandShakeError,
    WSStatusInvalidFrame,
    WSStatusConnectionCloseError,
    WSStatusDecodingFromUTF8Error,
    WSStatusIOError, 
} WSStatus;

typedef enum {
    WSREASON_SERVER_CLOSED,
    WSREASON_CLIENT_CLOSED
} WSReason;

typedef struct {
    WSReason reason;
    uint16_t status;
} WSReason_t;

typedef const void* RustEvent;

typedef enum WSEventKind { 
    WSEvent_CONNECT,
    WSEvent_TEXT,
    WSEvent_CLOSE,
} WSEventKind_t;

typedef struct WSEvent {
    WSEventKind_t kind;
    void* value; 
} WSEvent_t;

typedef struct {} WSSClient_t;

typedef void (*ws_handler_t)(WSSClient_t*, RustEvent, void*);

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
                    ws_handler_t callback);               

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
WSStatus wssclient_loop(WSSClient_t* client);


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
void wssclient_send(WSSClient_t* client, const char* message);


/*
* Drop the websocket from memory and close the connection with the server (graceful shutdown)
* 
* Parameters:
* - WSSClient_t* client
*
*/
void wssclient_drop(WSSClient_t* client);

WSEvent_t from_rust_event(RustEvent event);

#endif