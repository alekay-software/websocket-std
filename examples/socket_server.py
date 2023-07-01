from socket import socket, AF_INET, SOCK_STREAM

server = socket(AF_INET, SOCK_STREAM)
server.bind(("0.0.0.0", 3001))
server.listen(1)

try:
    client, address = server.accept()
    print("----------------------------------\n")
    msg = client.recv(1024)
    print(msg)
    print(msg.decode("utf-8"))
    print("\n----------------------------------\n")
    client.close()

except Exception as e:
    print("Error: ", e)
    pass

finally:
    client.close()
    server.close()