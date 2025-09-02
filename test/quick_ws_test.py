#!/usr/bin/env python3
"""
Quick WebSocket Performance Test
快速WebSocket性能测试
"""

import asyncio
import websockets
import json
import time
import random
import string
import base64
import io
import qrcode
from PIL import Image

async def quick_test(duration=10):
    """快速性能测试"""
    # 生成测试二维码
    def generate_test_qr():
        text = f"Test-{random.randint(1000, 9999)}-{''.join(random.choices(string.ascii_letters, k=10))}"
        qr = qrcode.QRCode(version=1, box_size=8, border=2)
        qr.add_data(text)
        qr.make(fit=True)
        img = qr.make_image(fill_color="black", back_color="white")
        img = img.resize((150, 150))
        
        buffer = io.BytesIO()
        img.save(buffer, format='PNG')
        return text, base64.b64encode(buffer.getvalue()).decode()
    
    print(f"🚀 Quick WebSocket FPS Test ({duration}s)")
    print("=" * 40)
    
    uri = "ws://localhost:3000/ws"
    sent = 0
    received = 0
    correct = 0
    start_time = time.time()
    
    try:
        async with websockets.connect(uri) as websocket:
            # 接收欢迎消息
            await websocket.recv()
            print("✅ Connected")
            
            while time.time() - start_time < duration:
                # 生成并发送二维码
                expected_text, qr_b64 = generate_test_qr()
                
                message = {"type": "detect", "image": qr_b64}
                await websocket.send(json.dumps(message))
                sent += 1
                
                # 接收响应
                response = await websocket.recv()
                received += 1
                
                result = json.loads(response)
                if result.get('success') and result.get('qrcodes'):
                    detected_texts = [qr.get('text', '') for qr in result['qrcodes']]
                    if expected_text in detected_texts:
                        correct += 1
                
                # 每100次显示进度
                if sent % 100 == 0:
                    elapsed = time.time() - start_time
                    fps = sent / elapsed
                    accuracy = (correct / received * 100) if received > 0 else 0
                    print(f"📊 {sent:4d} sent | {fps:5.1f} FPS | {accuracy:5.1f}% accuracy")
            
            # 关闭连接
            await websocket.send(json.dumps({"type": "close"}))
            await websocket.recv()
            
    except Exception as e:
        print(f"❌ Error: {e}")
        return
    
    # 统计结果
    elapsed = time.time() - start_time
    fps = sent / elapsed
    accuracy = (correct / received * 100) if received > 0 else 0
    
    print(f"\n🏁 Results:")
    print(f"   Time: {elapsed:.1f}s")
    print(f"   Sent: {sent}")
    print(f"   Received: {received}")
    print(f"   FPS: {fps:.1f}")
    print(f"   Accuracy: {accuracy:.1f}%")
    print(f"   Correct: {correct}/{received}")

if __name__ == "__main__":
    asyncio.run(quick_test())
