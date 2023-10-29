from socket import socket, AF_INET, SOCK_STREAM
import signal
import sys

server = socket(AF_INET, SOCK_STREAM)
server.bind(("localhost", 3000))
client = None

# Función que manejará la señal SIGINT (Ctrl+C)
def sigint_handler(signum, frame):
    print("Finishing")
    # Realiza las acciones que desees antes de salir
    # Por ejemplo, puedes guardar datos, cerrar archivos, liberar recursos, etc.
    if client is not None: client.close() 
    server.close()
    sys.exit(0)

# Registra el manejador de señal SIGINT
signal.signal(signal.SIGINT, sigint_handler)


server.listen(1)

print("Listening...")
while True:
    (client, address) = server.accept()
    print("New client: ", address)
    msg = client.recv(2048).decode("utf-8")
    print("Message: ", msg)
    client.close()

# if __name__ == "__main__":
