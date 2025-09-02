#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import requests
import time
import statistics
import threading
import json
import base64
from concurrent.futures import ThreadPoolExecutor, as_completed
import sys
from pathlib import Path

class QRCodeBenchmark:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
        self.test_image_path = "test_qr.png"
        self.results = []
        
    def check_service_health(self):
        """检查服务健康状态"""
        try:
            response = requests.get(f"{self.base_url}/health?verbose=true", timeout=5)
            if response.status_code == 200:
                health_data = response.json()
                print("✅ 服务状态正常")
                print(f"📊 服务信息: {health_data['service']} v{health_data['version']}")
                if 'pool_stats' in health_data:
                    pool_stats = health_data['pool_stats']
                    print(f"🏊 对象池配置: 初始大小={pool_stats['initial_size']}, 最大大小={pool_stats['max_size']}")
                print(f"🔧 特性: {', '.join(health_data.get('features', {}).keys())}")
                return True
            else:
                print(f"❌ 服务健康检查失败: HTTP {response.status_code}")
                return False
        except Exception as e:
            print(f"❌ 无法连接到服务: {e}")
            return False
    
    def load_test_image(self):
        """加载测试图片"""
        if not Path(self.test_image_path).exists():
            print(f"❌ 测试图片不存在: {self.test_image_path}")
            return None
        
        with open(self.test_image_path, 'rb') as f:
            return f.read()
    
    def single_request_file(self, image_data):
        """单次文件上传请求"""
        # 分阶段计时
        start_time = time.time()
        prepare_start = time.time()
        
        try:
            files = {'file': ('test.png', image_data, 'image/png')}
            prepare_time = time.time() - prepare_start
            
            request_start = time.time()
            response = requests.post(f"{self.base_url}/detect/file", files=files, timeout=30)
            request_time = time.time() - request_start
            
            parse_start = time.time()
            if response.status_code == 200:
                data = response.json()
                parse_time = time.time() - parse_start
                
                end_time = time.time()
                total_client_time = (end_time - start_time) * 1000
                
                return {
                    'success': True,
                    'total_time': total_client_time,  # ms
                    'prepare_time': prepare_time * 1000,  # 请求准备时间
                    'request_time': request_time * 1000,  # 网络+服务器时间
                    'parse_time': parse_time * 1000,     # 响应解析时间
                    'server_stats': data.get('statistics', {}),
                    'qr_count': data.get('count', 0),
                    'response_size': len(response.content)
                }
            else:
                end_time = time.time()
                return {
                    'success': False,
                    'total_time': (end_time - start_time) * 1000,
                    'error': f"HTTP {response.status_code}"
                }
        except Exception as e:
            end_time = time.time()
            return {
                'success': False,
                'total_time': (end_time - start_time) * 1000,
                'error': str(e)
            }
    
    def single_request_base64(self, image_data):
        """单次Base64请求"""
        start_time = time.time()
        try:
            b64_data = base64.b64encode(image_data).decode('utf-8')
            payload = {'image': b64_data}
            response = requests.post(f"{self.base_url}/detect/base64", 
                                   json=payload, timeout=30)
            end_time = time.time()
            
            if response.status_code == 200:
                data = response.json()
                return {
                    'success': True,
                    'total_time': (end_time - start_time) * 1000,
                    'server_stats': data.get('statistics', {}),
                    'qr_count': data.get('count', 0)
                }
            else:
                return {
                    'success': False,
                    'total_time': (end_time - start_time) * 1000,
                    'error': f"HTTP {response.status_code}"
                }
        except Exception as e:
            end_time = time.time()
            return {
                'success': False,
                'total_time': (end_time - start_time) * 1000,
                'error': str(e)
            }
    
    def warmup(self, image_data, warmup_count=10):
        """预热测试"""
        print(f"🔥 预热测试 ({warmup_count} 次请求)...")
        for i in range(warmup_count):
            self.single_request_file(image_data)
            print(f"   预热进度: {i+1}/{warmup_count}", end='\r')
        print("✅ 预热完成" + " " * 20)
    
    def benchmark_sequential(self, image_data, request_count=50):
        """串行基准测试"""
        print(f"\n📊 串行基准测试 ({request_count} 次请求)")
        print("-" * 60)
        
        results = []
        start_time = time.time()
        
        for i in range(request_count):
            result = self.single_request_file(image_data)
            results.append(result)
            print(f"   进度: {i+1}/{request_count} ({((i+1)/request_count*100):.1f}%)", end='\r')
        
        total_time = time.time() - start_time
        self.print_results("串行测试", results, total_time)
        return results
    
    def benchmark_concurrent(self, image_data, concurrent_count=10, total_requests=100):
        """并发压力测试"""
        print(f"\n🚀 并发压力测试 ({concurrent_count} 并发, {total_requests} 总请求)")
        print("-" * 60)
        
        results = []
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=concurrent_count) as executor:
            # 提交所有任务
            futures = [executor.submit(self.single_request_file, image_data) 
                      for _ in range(total_requests)]
            
            # 收集结果
            completed = 0
            for future in as_completed(futures):
                result = future.result()
                results.append(result)
                completed += 1
                print(f"   进度: {completed}/{total_requests} ({(completed/total_requests*100):.1f}%)", end='\r')
        
        total_time = time.time() - start_time
        self.print_results(f"并发测试 ({concurrent_count}并发)", results, total_time)
        return results
    
    def benchmark_mixed_api(self, image_data, concurrent_count=10, total_requests=100):
        """混合API测试（文件上传 + Base64）"""
        print(f"\n🔀 混合API测试 ({concurrent_count} 并发, {total_requests} 总请求)")
        print("-" * 60)
        
        results = []
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=concurrent_count) as executor:
            futures = []
            # 交替使用文件上传和Base64 API
            for i in range(total_requests):
                if i % 2 == 0:
                    futures.append(executor.submit(self.single_request_file, image_data))
                else:
                    futures.append(executor.submit(self.single_request_base64, image_data))
            
            completed = 0
            for future in as_completed(futures):
                result = future.result()
                results.append(result)
                completed += 1
                print(f"   进度: {completed}/{total_requests} ({(completed/total_requests*100):.1f}%)", end='\r')
        
        total_time = time.time() - start_time
        self.print_results("混合API测试", results, total_time)
        return results
    
    def sustained_load_test(self, image_data, concurrent_count=5, duration_seconds=30):
        """持续负载测试"""
        print(f"\n⏱️  持续负载测试 ({concurrent_count} 并发, {duration_seconds} 秒)")
        print("-" * 60)
        
        results = []
        start_time = time.time()
        end_time = start_time + duration_seconds
        
        def worker():
            while time.time() < end_time:
                result = self.single_request_file(image_data)
                results.append(result)
        
        threads = []
        for _ in range(concurrent_count):
            thread = threading.Thread(target=worker)
            threads.append(thread)
            thread.start()
        
        # 监控进度
        while time.time() < end_time:
            elapsed = time.time() - start_time
            remaining = duration_seconds - elapsed
            print(f"   运行中: {elapsed:.1f}s / {duration_seconds}s, 已完成: {len(results)} 请求", end='\r')
            time.sleep(1)
        
        # 等待所有线程完成
        for thread in threads:
            thread.join()
        
        total_time = time.time() - start_time
        self.print_results("持续负载测试", results, total_time)
        return results
    
    def print_results(self, test_name, results, total_time):
        """打印测试结果"""
        print("\n" + " " * 80)  # 清除进度显示
        
        successful_results = [r for r in results if r['success']]
        failed_count = len(results) - len(successful_results)
        
        if not successful_results:
            print(f"❌ {test_name}: 所有请求都失败了!")
            return
        
        # 计算统计数据
        client_times = [r['total_time'] for r in successful_results]
        server_times = [r['server_stats'].get('total_time_ms', 0) for r in successful_results if 'server_stats' in r]
        decode_times = [r['server_stats'].get('image_decode_time_ms', 0) for r in successful_results if 'server_stats' in r]
        detection_times = [r['server_stats'].get('detection_time_ms', 0) for r in successful_results if 'server_stats' in r]
        pool_times = [r['server_stats'].get('pool_acquisition_time_ms', 0) for r in successful_results if 'server_stats' in r]
        
        qps = len(successful_results) / total_time if total_time > 0 else 0
        
        print(f"📈 {test_name} 结果:")
        print(f"   总请求数: {len(results)}")
        print(f"   成功请求: {len(successful_results)}")
        print(f"   失败请求: {failed_count}")
        print(f"   总耗时: {total_time:.2f}s")
        print(f"   QPS: {qps:.2f}")
        
        if client_times:
            print(f"\n   客户端响应时间 (ms):")
            print(f"     平均: {statistics.mean(client_times):.2f}")
            print(f"     中位数: {statistics.median(client_times):.2f}")
            print(f"     最小: {min(client_times):.2f}")
            print(f"     最大: {max(client_times):.2f}")
            print(f"     95%: {self.percentile(client_times, 95):.2f}")
            print(f"     99%: {self.percentile(client_times, 99):.2f}")
            
            # 详细时间分解
            prepare_times = [r.get('prepare_time', 0) for r in successful_results if 'prepare_time' in r]
            request_times = [r.get('request_time', 0) for r in successful_results if 'request_time' in r]
            parse_times = [r.get('parse_time', 0) for r in successful_results if 'parse_time' in r]
            response_sizes = [r.get('response_size', 0) for r in successful_results if 'response_size' in r]
            
            if prepare_times and request_times and parse_times:
                print(f"\n   时间分解分析 (ms):")
                print(f"     请求准备时间: {statistics.mean(prepare_times):.3f}")
                print(f"     网络+服务器时间: {statistics.mean(request_times):.2f}")
                print(f"     响应解析时间: {statistics.mean(parse_times):.3f}")
                
                # 计算网络开销
                network_overhead = statistics.mean(request_times) - statistics.mean(server_times) if server_times else 0
                print(f"     网络+HTTP开销: {network_overhead:.2f}")
                
            if response_sizes:
                print(f"     平均响应大小: {statistics.mean(response_sizes):.0f} bytes")
        
        if server_times:
            print(f"\n   服务端处理时间 (ms):")
            print(f"     总时间平均: {statistics.mean(server_times):.2f}")
            print(f"     解码时间平均: {statistics.mean(decode_times):.2f}")
            print(f"     检测时间平均: {statistics.mean(detection_times):.2f}")
            if pool_times and any(t > 0 for t in pool_times):
                print(f"     对象池获取时间平均: {statistics.mean(pool_times):.3f}")
        
        if failed_count > 0:
            print(f"\n   ⚠️  失败原因统计:")
            failed_results = [r for r in results if not r['success']]
            error_counts = {}
            for r in failed_results:
                error = r.get('error', 'Unknown')
                error_counts[error] = error_counts.get(error, 0) + 1
            for error, count in error_counts.items():
                print(f"     {error}: {count}")
    
    def percentile(self, data, percentile):
        """计算百分位数"""
        size = len(data)
        return sorted(data)[int(size * percentile / 100)]
    
    def run_all_benchmarks(self):
        """运行所有基准测试"""
        print("=" * 80)
        print("🎯 QR码服务器性能基准测试")
        print("=" * 80)
        
        # 检查服务状态
        if not self.check_service_health():
            sys.exit(1)
        
        # 加载测试图片
        image_data = self.load_test_image()
        if image_data is None:
            sys.exit(1)
        
        print(f"📸 测试图片大小: {len(image_data)} bytes")
        
        # 预热
        self.warmup(image_data)
        
        # 各种基准测试
        self.benchmark_sequential(image_data, 20)
        self.benchmark_concurrent(image_data, 5, 50)
        self.benchmark_concurrent(image_data, 10, 100)
        self.benchmark_concurrent(image_data, 20, 200)
        self.benchmark_mixed_api(image_data, 10, 100)
        self.sustained_load_test(image_data, 10, 20)
        
        print("\n" + "=" * 80)
        print("✨ 所有基准测试完成!")
        print("=" * 80)

if __name__ == "__main__":
    benchmark = QRCodeBenchmark()
    benchmark.run_all_benchmarks()
