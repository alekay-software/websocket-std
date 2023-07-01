#!/usr/bin/env python
import asyncio
from websockets.server import serve

async def echo(websocket):
    async for message in websocket:
        print("New message: ", message)
        await websocket.send("Hello from server")

async def main():
    async with serve(echo, "127.0.0.1", 3000):
        await asyncio.Future()  # run forever

asyncio.run(main())