# from requests import get

# response = get("http://localhost:3000/test")

# print(response.)

from socket import socket, AF_INET, SOCK_STREAM

s = socket(AF_INET, SOCK_STREAM)
s.connect(("localhost", 3000))

request = "GET /test HTTP/1.1\r\nHost: 127.0.0.1:3000\r\n\r\n"

s.send(request.encode("utf-8"))

response = s.recv(2048)

# print(response)
# print("\n\n")

# response.replace("\r\n", "\n")

print(response.decode("utf-8"))
print("--------------------------")
print(response)