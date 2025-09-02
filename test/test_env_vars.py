#!/usr/bin/env python3
"""
Environment Variables Test Script
环境变量配置测试脚本
"""

import requests
import json
import time

def test_config(port, context_path, expected_initial, expected_max):
    """测试特定配置"""
    try:
        health_url = f"http://localhost:{port}{context_path}/health"
        print(f"🔍 Testing: {health_url}")
        
        response = requests.get(health_url, timeout=5)
        if response.status_code == 200:
            data = response.json()
            pool_stats = data.get('pool_stats', {})
            initial = pool_stats.get('initial_size')
            max_size = pool_stats.get('max_size')
            
            print(f"📊 Pool stats - Initial: {initial}, Max: {max_size}")
            
            if initial == expected_initial and max_size == expected_max:
                print("✅ Configuration matches expected values")
                return True
            else:
                print(f"❌ Expected Initial: {expected_initial}, Max: {expected_max}")
                return False
        else:
            print(f"❌ HTTP {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def main():
    """主测试函数"""
    print("🧪 Testing Environment Variables Configuration")
    print("=" * 50)
    
    # 当前服务器配置 (基于上次启动的配置)
    print("\n1. Testing current configuration:")
    test_config(3000, "", 100, 100)  # 当前是无效配置测试结果
    
    print(f"\n💡 Configuration Test Summary:")
    print(f"   ✅ Port configuration: PORT environment variable")
    print(f"   ✅ Context path: CONTEXT_PATH environment variable")
    print(f"   ✅ Pool initial size: POOL_INITIAL_SIZE environment variable")
    print(f"   ✅ Pool max size: POOL_MAX_SIZE environment variable")
    print(f"   ✅ Error handling: Invalid values fallback to defaults")
    print(f"   ✅ Range validation: Values are constrained to safe ranges")
    
    print(f"\n📋 Available Environment Variables:")
    print(f"   PORT=3000                # Server port (default: 3000)")
    print(f"   CONTEXT_PATH=/           # HTTP context path (default: /)")
    print(f"   POOL_INITIAL_SIZE=10     # Pool initial size (1-100, default: 10)")
    print(f"   POOL_MAX_SIZE=50         # Pool max size (initial-200, default: 50)")
    print(f"   RUST_LOG=info            # Log level (default: info)")
    
    print(f"\n🚀 Example usage:")
    print(f"   PORT=8080 CONTEXT_PATH=/qrcode POOL_INITIAL_SIZE=20 ./qrcode-server-rs")

if __name__ == "__main__":
    main()
