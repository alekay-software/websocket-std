#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum ConnectionStatus {
  NOT_INIT,
  HANDSHAKE,
  OPEN,
  CLIENT_WANTS_TO_CLOSE,
  SERVER_WANTS_TO_CLOSE,
  CLOSE,
} ConnectionStatus;

typedef struct Option_String Option_String;

typedef struct Option_TcpStream Option_TcpStream;

typedef struct Option_WSData_c_void Option_WSData_c_void;

typedef struct String String;

typedef struct VecDeque_Event VecDeque_Event;

typedef struct Vec_Extension Vec_Extension;

typedef struct Vec_u8 Vec_u8;

typedef enum Reason_Tag {
  SERVER_CLOSE,
  CLIENT_CLOSE,
} Reason_Tag;

typedef struct Reason {
  Reason_Tag tag;
  union {
    struct {
      uint16_t server_close;
    };
    struct {
      uint16_t client_close;
    };
  };
} Reason;

typedef enum WSEvent_Tag {
  ON_CONNECT,
  ON_CLOSE,
  ON_TEXT,
} WSEvent_Tag;

typedef struct WSEvent {
  WSEvent_Tag tag;
  union {
    struct {
      struct Reason on_close;
    };
    struct {
      struct String on_text;
    };
  };
} WSEvent;

typedef struct SyncClient_c_void {
  const str *host;
  uint16_t port;
  const str *path;
  enum ConnectionStatus connection_status;
  uint64_t message_size;
  Duration timeout;
  struct Option_TcpStream stream;
  struct Vec_u8 recv_storage;
  struct Vec_u8 recv_data;
  struct Option_WSData_c_void cb_data;
  void (*callback)(SyncClient*, struct WSEvent, struct Option_WSData_c_void);
  struct Option_String protocol;
  struct Vec_Extension extensions;
  struct VecDeque_Event input_events;
  struct VecDeque_Event output_events;
  struct String websocket_key;
  uintptr_t close_iters;
} SyncClient_c_void;

typedef struct Host {
  const str *host;
} Host;

CSyncClient *syncClientNew(void);

void syncClientInit(struct SyncClient_c_void *client,
                    const char *host,
                    uint16_t port,
                    const char *path,
                    void *callback);

struct Host hostNew(const char *host);

const char *getHost(const struct Host *host);
