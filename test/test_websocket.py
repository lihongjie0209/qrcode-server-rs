#!/usr/bin/env python3
"""
WebSocket QR Code Detection Test Script
测试WebSocket接口的QR码检测功能
"""

import asyncio
import websockets
import base64
import json
import time
import os
from pathlib import Path

class QRCodeWebSocketTester:
    def __init__(self, server_url="ws://localhost:3000/ws"):
        self.server_url = server_url
        self.test_images = []
        self.load_test_images()

    def load_test_images(self):
        """加载测试图片"""
        # 简单生成一个测试QR码的base64字符串
        # 这里使用一个包含简单QR码的小图片的base64
        # 实际使用时可以替换为真实的QR码图片
        test_qr_base64 = """
        iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFklEQVR42mNk+M/AAEIQCFg4zQjEgAAA
        VgAKpMUj8wAAAABJRU5ErkJggg==
        """
        self.test_images.append(test_qr_base64.replace('\n', '').replace(' ', ''))

    async def test_single_image(self):
        """测试发送单张图片"""
        print("🔍 Testing single image detection...")
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                print(f"✅ Connected to {self.server_url}")
                
                # 发送图片
                message = {
                    "type": "detect",
                    "image": self.test_images[0]
                }
                await websocket.send(json.dumps(message))
                print("📤 Sent image data")
                
                # 接收响应
                response = await websocket.recv()
                result = json.loads(response)
                print(f"📥 Received response: {result}")
                
                # 发送结束消息
                end_message = {"type": "close"}
                await websocket.send(json.dumps(end_message))
                print("🔚 Sent end message")
                
        except Exception as e:
            print(f"❌ Error in single image test: {e}")

    async def test_multiple_images(self, count=5):
        """测试发送多张图片"""
        print(f"🔍 Testing multiple images detection ({count} images)...")
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                print(f"✅ Connected to {self.server_url}")
                
                # 发送多张图片
                for i in range(count):
                    message = {
                        "type": "detect",
                        "image": self.test_images[0]  # 使用相同的测试图片
                    }
                    await websocket.send(json.dumps(message))
                    print(f"📤 Sent image {i+1}/{count}")
                    
                    # 接收响应
                    response = await websocket.recv()
                    result = json.loads(response)
                    print(f"📥 Response {i+1}: {result}")
                    
                    # 短暂延迟
                    await asyncio.sleep(0.1)
                
                # 发送结束消息
                end_message = {"type": "close"}
                await websocket.send(json.dumps(end_message))
                print("🔚 Sent end message")
                
        except Exception as e:
            print(f"❌ Error in multiple images test: {e}")

    async def test_performance(self, duration=10):
        """性能测试 - 在指定时间内持续发送图片"""
        print(f"🚀 Performance test for {duration} seconds...")
        
        start_time = time.time()
        image_count = 0
        success_count = 0
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                print(f"✅ Connected to {self.server_url}")
                
                while time.time() - start_time < duration:
                    try:
                        # 发送图片
                        message = {
                            "image": self.test_images[0]
                        }
                        await websocket.send(json.dumps(message))
                        image_count += 1
                        
                        # 接收响应
                        response = await websocket.recv()
                        result = json.loads(response)
                        success_count += 1
                        
                        # 显示进度
                        if image_count % 10 == 0:
                            elapsed = time.time() - start_time
                            rate = image_count / elapsed
                            print(f"📊 Processed {image_count} images, rate: {rate:.1f} img/s")
                        
                    except Exception as e:
                        print(f"❌ Error during performance test: {e}")
                        break
                
                # 发送结束消息
                end_message = {"end": True}
                await websocket.send(json.dumps(end_message))
                
                # 计算统计信息
                elapsed_time = time.time() - start_time
                success_rate = success_count / image_count * 100 if image_count > 0 else 0
                avg_rate = image_count / elapsed_time
                
                print(f"\n📈 Performance Results:")
                print(f"   Total images sent: {image_count}")
                print(f"   Successful responses: {success_count}")
                print(f"   Success rate: {success_rate:.1f}%")
                print(f"   Average rate: {avg_rate:.1f} images/second")
                print(f"   Total time: {elapsed_time:.1f} seconds")
                
        except Exception as e:
            print(f"❌ Error in performance test: {e}")

    async def test_connection_handling(self):
        """测试连接处理"""
        print("🔗 Testing connection handling...")
        
        try:
            # 测试正常连接和断开
            async with websockets.connect(self.server_url) as websocket:
                print("✅ Connected successfully")
                
                # 测试ping
                await websocket.ping()
                print("🏓 Ping successful")
                
                # 发送一条消息
                message = {"image": self.test_images[0]}
                await websocket.send(json.dumps(message))
                response = await websocket.recv()
                print("📤📥 Message exchange successful")
                
                # 正常关闭
                end_message = {"end": True}
                await websocket.send(json.dumps(end_message))
                print("🔚 Normal closure successful")
                
        except Exception as e:
            print(f"❌ Error in connection handling test: {e}")

    async def test_invalid_data(self):
        """测试无效数据处理"""
        print("🚨 Testing invalid data handling...")
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                print("✅ Connected")
                
                # 测试无效JSON
                try:
                    await websocket.send("invalid json")
                    response = await websocket.recv()
                    print(f"📥 Response to invalid JSON: {response}")
                except Exception as e:
                    print(f"⚠️ Invalid JSON handled: {e}")
                
                # 测试缺少字段的JSON
                try:
                    await websocket.send(json.dumps({"wrong_field": "value"}))
                    response = await websocket.recv()
                    print(f"📥 Response to wrong field: {response}")
                except Exception as e:
                    print(f"⚠️ Wrong field handled: {e}")
                
                # 测试无效base64
                try:
                    await websocket.send(json.dumps({"image": "invalid_base64"}))
                    response = await websocket.recv()
                    print(f"📥 Response to invalid base64: {response}")
                except Exception as e:
                    print(f"⚠️ Invalid base64 handled: {e}")
                
                # 正常结束
                end_message = {"end": True}
                await websocket.send(json.dumps(end_message))
                
        except Exception as e:
            print(f"❌ Error in invalid data test: {e}")

async def main():
    """主测试函数"""
    print("🚀 QR Code WebSocket API Test Suite")
    print("=" * 50)
    
    tester = QRCodeWebSocketTester()
    
    # 等待服务器启动
    print("⏳ Waiting for server to be ready...")
    await asyncio.sleep(2)
    
    try:
        # 基础功能测试
        await tester.test_connection_handling()
        print("\n" + "=" * 50)
        
        await tester.test_single_image()
        print("\n" + "=" * 50)
        
        await tester.test_multiple_images(3)
        print("\n" + "=" * 50)
        
        # 错误处理测试
        await tester.test_invalid_data()
        print("\n" + "=" * 50)
        
        # 性能测试
        await tester.test_performance(5)
        print("\n" + "=" * 50)
        
        print("✅ All tests completed!")
        
    except Exception as e:
        print(f"❌ Test suite error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
