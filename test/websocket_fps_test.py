#!/usr/bin/env python3
"""
WebSocket FPS Performance Test with Random QR Codes
使用随机生成的二维码测试WebSocket接口的FPS性能
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

class QRCodeGenerator:
    def __init__(self):
        self.qr_cache = []
        self.cache_size = 20
        self.generate_cache()
    
    def generate_random_text(self, length=20):
        """生成随机文本"""
        chars = string.ascii_letters + string.digits + "!@#$%^&*()_+-=[]{}|;:,.<>?"
        return ''.join(random.choice(chars) for _ in range(length))
    
    def generate_qr_image(self, text, size=(200, 200)):
        """生成二维码图片"""
        qr = qrcode.QRCode(
            version=1,
            error_correction=qrcode.constants.ERROR_CORRECT_L,
            box_size=10,
            border=4,
        )
        qr.add_data(text)
        qr.make(fit=True)
        
        # 创建图片
        img = qr.make_image(fill_color="black", back_color="white")
        img = img.resize(size, Image.Resampling.LANCZOS)
        
        # 转换为base64
        buffer = io.BytesIO()
        img.save(buffer, format='PNG')
        img_str = base64.b64encode(buffer.getvalue()).decode()
        
        return text, img_str
    
    def generate_cache(self):
        """预生成二维码缓存"""
        print(f"🔄 Generating {self.cache_size} QR codes for testing...")
        self.qr_cache = []
        for i in range(self.cache_size):
            text = f"Test QR {i}: {self.generate_random_text()}"
            _, img_b64 = self.generate_qr_image(text)
            self.qr_cache.append((text, img_b64))
        print("✅ QR code cache generated")
    
    def get_random_qr(self):
        """获取随机二维码"""
        return random.choice(self.qr_cache)

class WebSocketFPSTest:
    def __init__(self, server_url="ws://localhost:3000/ws"):
        self.server_url = server_url
        self.qr_generator = QRCodeGenerator()
        
    async def test_fps_continuous(self, duration=10):
        """连续发送测试 - 测量FPS"""
        print(f"🚀 Starting WebSocket FPS test for {duration} seconds...")
        
        start_time = time.time()
        sent_count = 0
        received_count = 0
        success_count = 0
        error_count = 0
        
        response_times = []
        detection_results = []
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                print("✅ Connected to WebSocket")
                
                # 接收欢迎消息
                welcome = await websocket.recv()
                print("📥 Welcome message received")
                
                while time.time() - start_time < duration:
                    # 获取随机二维码
                    expected_text, qr_image = self.qr_generator.get_random_qr()
                    
                    # 记录发送时间
                    send_time = time.time()
                    
                    # 发送检测请求
                    message = {
                        "type": "detect",
                        "image": qr_image
                    }
                    await websocket.send(json.dumps(message))
                    sent_count += 1
                    
                    # 接收响应
                    response = await websocket.recv()
                    receive_time = time.time()
                    received_count += 1
                    
                    # 计算响应时间
                    response_time = (receive_time - send_time) * 1000  # ms
                    response_times.append(response_time)
                    
                    # 解析响应
                    try:
                        result = json.loads(response)
                        if result.get('success', False):
                            success_count += 1
                            qrcodes = result.get('qrcodes', [])
                            
                            # 验证检测结果
                            detected_texts = [qr.get('text', '') for qr in qrcodes]
                            if expected_text in detected_texts:
                                detection_results.append({'expected': expected_text, 'detected': True})
                            else:
                                detection_results.append({'expected': expected_text, 'detected': False, 'found': detected_texts})
                        else:
                            error_count += 1
                    except Exception as e:
                        error_count += 1
                        print(f"❌ Response parse error: {e}")
                    
                    # 显示进度
                    if sent_count % 10 == 0:
                        elapsed = time.time() - start_time
                        current_fps = sent_count / elapsed
                        print(f"📊 Progress: {sent_count} sent, {received_count} received, {current_fps:.1f} FPS")
                
                # 发送关闭消息
                close_message = {"type": "close"}
                await websocket.send(json.dumps(close_message))
                close_response = await websocket.recv()
                
        except Exception as e:
            print(f"❌ WebSocket error: {e}")
        
        # 计算统计信息
        elapsed_time = time.time() - start_time
        avg_fps = sent_count / elapsed_time
        receive_fps = received_count / elapsed_time
        success_rate = success_count / received_count * 100 if received_count > 0 else 0
        
        # 计算响应时间统计
        if response_times:
            avg_response_time = sum(response_times) / len(response_times)
            min_response_time = min(response_times)
            max_response_time = max(response_times)
        else:
            avg_response_time = min_response_time = max_response_time = 0
        
        # 计算检测准确率
        correct_detections = sum(1 for r in detection_results if r.get('detected', False))
        detection_accuracy = correct_detections / len(detection_results) * 100 if detection_results else 0
        
        print(f"\n🏁 WebSocket FPS Test Results:")
        print(f"=" * 50)
        print(f"📊 Performance Metrics:")
        print(f"   Duration: {elapsed_time:.1f} seconds")
        print(f"   Messages sent: {sent_count}")
        print(f"   Responses received: {received_count}")
        print(f"   Send FPS: {avg_fps:.1f}")
        print(f"   Receive FPS: {receive_fps:.1f}")
        print(f"   Success rate: {success_rate:.1f}%")
        print(f"   Error count: {error_count}")
        print(f"\n⏱️  Response Time Statistics:")
        print(f"   Average: {avg_response_time:.1f} ms")
        print(f"   Minimum: {min_response_time:.1f} ms")
        print(f"   Maximum: {max_response_time:.1f} ms")
        print(f"\n🎯 Detection Accuracy:")
        print(f"   Correct detections: {correct_detections}/{len(detection_results)}")
        print(f"   Accuracy rate: {detection_accuracy:.1f}%")
        
        return {
            'fps': avg_fps,
            'receive_fps': receive_fps,
            'success_rate': success_rate,
            'avg_response_time': avg_response_time,
            'detection_accuracy': detection_accuracy
        }
    
    async def test_burst_mode(self, burst_size=50):
        """突发模式测试 - 快速发送多个请求"""
        print(f"💥 Burst mode test: sending {burst_size} requests rapidly...")
        
        start_time = time.time()
        
        try:
            async with websockets.connect(self.server_url) as websocket:
                # 接收欢迎消息
                await websocket.recv()
                
                # 快速发送多个请求
                send_times = []
                for i in range(burst_size):
                    _, qr_image = self.qr_generator.get_random_qr()
                    message = {
                        "type": "detect", 
                        "image": qr_image
                    }
                    
                    send_start = time.time()
                    await websocket.send(json.dumps(message))
                    send_times.append(time.time() - send_start)
                
                send_duration = time.time() - start_time
                
                # 接收所有响应
                receive_start = time.time()
                responses = []
                for i in range(burst_size):
                    response = await websocket.recv()
                    responses.append(response)
                
                receive_duration = time.time() - receive_start
                total_duration = time.time() - start_time
                
                # 发送关闭消息
                await websocket.send(json.dumps({"type": "close"}))
                await websocket.recv()
                
                print(f"📊 Burst Test Results:")
                print(f"   Sent {burst_size} requests in {send_duration:.2f}s")
                print(f"   Send rate: {burst_size/send_duration:.1f} req/s")
                print(f"   Received {len(responses)} responses in {receive_duration:.2f}s")
                print(f"   Receive rate: {len(responses)/receive_duration:.1f} resp/s")
                print(f"   Total time: {total_duration:.2f}s")
                print(f"   Overall throughput: {burst_size/total_duration:.1f} req/s")
                
        except Exception as e:
            print(f"❌ Burst test error: {e}")

async def main():
    """主测试函数"""
    print("🚀 WebSocket FPS Performance Test with Random QR Codes")
    print("=" * 60)
    
    tester = WebSocketFPSTest()
    
    # 等待服务器准备就绪
    print("⏳ Waiting for server...")
    await asyncio.sleep(2)
    
    try:
        # 短时间性能测试
        print("\n🔥 Quick Performance Test (5 seconds)")
        print("-" * 40)
        await tester.test_fps_continuous(5)
        
        await asyncio.sleep(1)
        
        # 中等时长性能测试
        print("\n🏃 Medium Performance Test (15 seconds)")
        print("-" * 40)
        results = await tester.test_fps_continuous(15)
        
        await asyncio.sleep(1)
        
        # 突发模式测试
        print("\n💥 Burst Mode Test")
        print("-" * 40)
        await tester.test_burst_mode(30)
        
        print(f"\n✅ All performance tests completed!")
        print(f"🏆 Best sustained FPS: {results['fps']:.1f}")
        
    except Exception as e:
        print(f"❌ Test suite error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
