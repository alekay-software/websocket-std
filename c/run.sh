# Copy static lib
cp ../target/debug/libwebsocket_std.a c/websocket

# Compile and run main.c
clang -o main main.c -L websocket -lwebsocket_std && ./main