#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import requests
import time
import statistics
import json

def detailed_timing_analysis():
    """详细的时间分析"""
    print("🔍 详细时间分析测试")
    print("=" * 50)
    
    # 加载测试图片
    with open("test_qr.png", 'rb') as f:
        image_data = f.read()
    
    results = []
    
    # 进行10次测试
    for i in range(10):
        print(f"测试 {i+1}/10...", end=' ')
        
        # 详细计时
        total_start = time.time()
        
        # 1. 准备请求
        prepare_start = time.time()
        files = {'file': ('test.png', image_data, 'image/png')}
        prepare_time = (time.time() - prepare_start) * 1000
        
        # 2. 发送请求和接收响应
        request_start = time.time()
        response = requests.post("http://localhost:3000/detect/file", files=files)
        request_time = (time.time() - request_start) * 1000
        
        # 3. 解析响应
        parse_start = time.time()
        data = response.json()
        parse_time = (time.time() - parse_start) * 1000
        
        total_time = (time.time() - total_start) * 1000
        
        server_stats = data.get('statistics', {})
        
        result = {
            'total_client_time': total_time,
            'prepare_time': prepare_time,
            'request_time': request_time, 
            'parse_time': parse_time,
            'server_total_time': server_stats.get('total_time_ms', 0),
            'server_decode_time': server_stats.get('image_decode_time_ms', 0),
            'server_detection_time': server_stats.get('detection_time_ms', 0),
            'server_pool_time': server_stats.get('pool_acquisition_time_ms', 0),
            'response_size': len(response.content)
        }
        
        results.append(result)
        print(f"总时间: {total_time:.1f}ms, 服务器: {result['server_total_time']:.1f}ms")
    
    # 统计分析
    print("\n📊 时间分解统计:")
    print("-" * 50)
    
    metrics = [
        ('客户端总时间', 'total_client_time'),
        ('请求准备时间', 'prepare_time'), 
        ('网络+服务器时间', 'request_time'),
        ('响应解析时间', 'parse_time'),
        ('服务器总时间', 'server_total_time'),
        ('服务器解码时间', 'server_decode_time'),
        ('服务器检测时间', 'server_detection_time'),
        ('对象池获取时间', 'server_pool_time')
    ]
    
    for name, key in metrics:
        values = [r[key] for r in results]
        if any(v > 0 for v in values):
            avg = statistics.mean(values)
            print(f"{name:15}: {avg:8.3f}ms")
    
    # 计算网络开销
    client_request_avg = statistics.mean([r['request_time'] for r in results])
    server_total_avg = statistics.mean([r['server_total_time'] for r in results])
    network_overhead = client_request_avg - server_total_avg
    
    print(f"\n🌐 网络+HTTP开销分析:")
    print(f"网络+服务器时间: {client_request_avg:.2f}ms")
    print(f"服务器处理时间:   {server_total_avg:.2f}ms") 
    print(f"网络+HTTP开销:    {network_overhead:.2f}ms ({network_overhead/client_request_avg*100:.1f}%)")
    
    # 响应大小
    avg_response_size = statistics.mean([r['response_size'] for r in results])
    print(f"平均响应大小:     {avg_response_size:.0f} bytes")

if __name__ == "__main__":
    detailed_timing_analysis()
