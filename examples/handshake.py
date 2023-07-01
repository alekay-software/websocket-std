from socket import socket, AF_INET, SOCK_STREAM
from random import randbytes, getrandbits
from secrets import token_hex
from typing import List

def reverse_binary(binary: str) -> str:
    b = binary[2:][::-1]
    return "0b" + b

def get_headers(host: str, port: int, path: str) -> str:
    END_LINE = "\r\n"
    headers = f"GET {path} HTTP/1.1" + END_LINE
    headers += f"Host: {host}:{port}" + END_LINE 
    headers += "Upgrade: websocket" + END_LINE
    headers += "Connection: upgrade" + END_LINE
    headers += "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" + END_LINE
    headers += f"Sec-WebSocket-Version: 13" + END_LINE

    return headers


def mask_payload(payload: bytes, mask: List[int]) -> bytes:
    i = 0
    masked_payload = bytearray()
    _mask = bytearray(mask)
    _payload = bytearray(payload)
    for b in _payload:
        _a = b
        _b = _mask[i]
        masked_payload.append(_a ^ _b) # XOR 
        i += 1
        if i >= 4: i = 0
    return masked_payload


def get_mask() -> int:
    return token_hex(4)

def send_single_message(socket: socket, payload: str):
    if len(payload) > 125:
        print("Cannot send a message greater than 125 characters")
    
    frame = bytearray()
    frame.append(0b10000001)      # FIN, RSV1, RSV2, RSV3, Opcode for text message
    frame.append(0b10000101)   # Enable mask because is a client-server message and single payload length
    mask1 = 0b11111111
    mask2 = 0b11111111
    mask3 = 0b00000000
    mask4 = 0b00000000
    frame.append(mask1)                          # Mask
    frame.append(mask2)                          # Mask
    frame.append(mask3)                          # Mask
    frame.append(mask4)                          # Mask                          
    frame += (mask_payload("hola".encode("utf-8"), [mask1, mask2, mask3, mask4]))

    print(frame)

    socket.send(frame)

def send_message_old(socket: socket, payload: str):
    # frame = "0b1000" # Fin, RSV1, RSV2, RSV3
    # frame += "0b0001" # opcode for text frame
    # frame += "0b0" # Mask (no mask)
    # frame += bin(len(payload))
    # frame += "0b0000000000000000" # No extended payload
    # frame += "0b00000000000000000000000000000000" # No extended payload
    # frame += "0b0000000000000000" # No extended payload
    # # frame += "0b00000000000000000000000000000000" # No mask

    # frame = frame.encode("utf-8")ksj
    # frame += payload.encode("utf-8") # Insert payloadksj

    # socket.send(frame)

    # A single-frame unmasked text message (contains hello)
    frame = bytearray([0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58])
    # Binary representation
    # 10000001 hex(81) -> int(129)
    # 00000101 hex(05) -> int(5)
    # 01001000 hex(48) -> int(72)
    # 01100101 hex(65) -> int(101)
    # 01101100 hex(6c) -> int(108)
    # 01101100 hex(6c) -> int(108)
    # 01101111 hex(6f) -> int(111)

    socket.send(frame)

def send_close(socket: socket):
    frame = bytearray([0x81, 0x88, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58])
    socket.send(frame)

SERVER = "localhost"
PORT = 3000
PATH = "/"

client = socket(AF_INET, SOCK_STREAM)

client.connect((SERVER, PORT))

headers = get_headers(SERVER, PORT, PATH)

headers = "GET / HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"

print("-------------- HEADERS ------------------")
print(headers)

client.send(headers.encode("utf-8"))

# Show handshacke response
print(client.recv(2048).decode("utf-8"))

send_single_message(client, "hello from scracth python client")
send_close(client)

# Finally close the connection
client.close()