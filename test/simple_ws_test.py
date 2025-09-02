#!/usr/bin/env python3
"""
Simple WebSocket QR Code Test
简单的WebSocket QR码测试脚本
"""

import asyncio
import websockets
import json
import base64

# 创建一个简单的测试图片 (1x1像素的PNG)
TEST_IMAGE_BASE64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="

async def test_websocket():
    uri = "ws://localhost:3000/ws"
    
    try:
        print(f"🔗 Connecting to {uri}...")
        async with websockets.connect(uri) as websocket:
            print("✅ Connected successfully!")
            
            # 接收欢迎消息
            print("⏳ Waiting for welcome message...")
            welcome_response = await websocket.recv()
            welcome_result = json.loads(welcome_response)
            print(f"📥 Welcome: {welcome_result}")
            
            # 发送测试图片
            message = {
                "type": "detect",
                "image": TEST_IMAGE_BASE64
            }
            print("📤 Sending test image...")
            await websocket.send(json.dumps(message))
            
            # 接收QR码检测响应
            print("⏳ Waiting for detection response...")
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Detection result: {result}")
            
            # 发送结束消息
            print("🔚 Sending end message...")
            end_message = {"type": "close"}
            await websocket.send(json.dumps(end_message))
            
            # 接收关闭确认
            close_response = await websocket.recv()
            close_result = json.loads(close_response)
            print(f"📥 Close confirmation: {close_result}")
            
            print("✅ Test completed successfully!")
            
    except websockets.exceptions.ConnectionRefused:
        print("❌ Connection refused. Make sure the server is running on localhost:3000")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_websocket())
