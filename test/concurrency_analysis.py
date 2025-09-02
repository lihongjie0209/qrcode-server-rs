#!/usr/bin/env python3

import requests
import time
import threading
import statistics
from concurrent.futures import ThreadPoolExecutor

def test_concurrency_impact():
    """测试并发对响应时间的影响"""
    print("🔬 并发对响应时间影响分析")
    print("=" * 60)
    
    with open("test_qr.png", 'rb') as f:
        image_data = f.read()
    
    def single_request():
        start_time = time.time()
        files = {'file': ('test.png', image_data, 'image/png')}
        response = requests.post("http://localhost:3000/detect/file", files=files)
        client_time = (time.time() - start_time) * 1000
        
        data = response.json()
        server_time = data.get('statistics', {}).get('total_time_ms', 0)
        
        return {
            'client_time': client_time,
            'server_time': server_time,
            'overhead': client_time - server_time
        }
    
    # 测试不同并发级别
    concurrency_levels = [1, 2, 5, 10, 15, 20]
    
    for concurrency in concurrency_levels:
        print(f"\n🚀 测试并发级别: {concurrency}")
        print("-" * 40)
        
        results = []
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            futures = [executor.submit(single_request) for _ in range(concurrency * 3)]
            for future in futures:
                results.append(future.result())
        
        total_time = time.time() - start_time
        
        client_times = [r['client_time'] for r in results]
        server_times = [r['server_time'] for r in results]
        overheads = [r['overhead'] for r in results]
        
        print(f"请求数量: {len(results)}")
        print(f"总耗时: {total_time:.2f}s")
        print(f"QPS: {len(results)/total_time:.1f}")
        print(f"客户端时间: {statistics.mean(client_times):.1f}ms (中位数: {statistics.median(client_times):.1f}ms)")
        print(f"服务器时间: {statistics.mean(server_times):.1f}ms")
        print(f"网络开销: {statistics.mean(overheads):.1f}ms ({statistics.mean(overheads)/statistics.mean(client_times)*100:.1f}%)")

if __name__ == "__main__":
    test_concurrency_impact()
