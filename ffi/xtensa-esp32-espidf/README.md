# websocket-std with ESP32

Here you will find the instructions to link the library in different projects.

## PlatformIO

- Create the folder ``websocket`` inside ``lib`` and copy the static library **libwebsocket_std.a** in the websocket folder.
- Copy the header file ``websocket-std.h`` inside include folder.
- Add the build flags in ``platformio.in``

```ini
build_flags = 
    -I ./include
    -L ./lib/websocket
    -lwebsocket_std
```