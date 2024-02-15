Implementation of websocket for ESP32 written in rust with std support.

1. Implement header, frame and mask from ws_basics.
2. Implement gen_key function to generate a random key (use rand module)
3. Use basics to build the syn client version of the websockets.
4. Try to implement async client without tokio. Implementing an event loop,     calling a loop method at the begining of an infinite loop (version similar to websockets of C implementation)
5. use thread to create an event loop.