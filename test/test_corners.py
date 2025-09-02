#!/usr/bin/env python3

import requests
import json

def test_corner_points():
    """测试角点定位准确性"""
    print("🎯 QR码角点定位测试")
    print("=" * 50)
    
    # 测试文件上传
    with open("test_qr.png", 'rb') as f:
        files = {'file': ('test.png', f.read(), 'image/png')}
    
    response = requests.post("http://localhost:3000/detect/file", files=files)
    data = response.json()
    
    if data['success'] and data['qrcodes']:
        qr = data['qrcodes'][0]
        
        print(f"📸 图片尺寸: {data['statistics']['image_width']} x {data['statistics']['image_height']}")
        print(f"🔍 检测到的QR码: {qr['text']}")
        print()
        
        print("📍 角点坐标:")
        for i, point in enumerate(qr['points']):
            print(f"  角点 {i+1}: ({point[0]:.2f}, {point[1]:.2f})")
        
        print()
        print("📐 边界框:")
        bbox = qr['bbox']
        print(f"  左上角: ({bbox['x']:.2f}, {bbox['y']:.2f})")
        print(f"  宽度: {bbox['width']:.2f}")
        print(f"  高度: {bbox['height']:.2f}")
        print(f"  右下角: ({bbox['x'] + bbox['width']:.2f}, {bbox['y'] + bbox['height']:.2f})")
        
        # 验证角点合理性
        print()
        print("✅ 角点合理性检查:")
        
        img_width = data['statistics']['image_width']
        img_height = data['statistics']['image_height']
        
        # 检查所有角点是否在图像范围内
        all_in_bounds = True
        for i, point in enumerate(qr['points']):
            x, y = point
            if x < 0 or x > img_width or y < 0 or y > img_height:
                print(f"  ❌ 角点 {i+1} 超出图像边界")
                all_in_bounds = False
        
        if all_in_bounds:
            print("  ✅ 所有角点都在图像范围内")
        
        # 检查QR码尺寸合理性
        qr_width = bbox['width']
        qr_height = bbox['height']
        area_ratio = (qr_width * qr_height) / (img_width * img_height)
        
        print(f"  📏 QR码占图像面积比: {area_ratio:.1%}")
        
        if 0.1 <= area_ratio <= 1.0:
            print("  ✅ QR码尺寸合理")
        else:
            print("  ⚠️  QR码尺寸可能异常")
        
        # 检查角点顺序（应该是顺时针或逆时针）
        print()
        print("🔄 角点顺序分析:")
        
        # 计算质心
        center_x = sum(p[0] for p in qr['points']) / 4
        center_y = sum(p[1] for p in qr['points']) / 4
        print(f"  质心: ({center_x:.2f}, {center_y:.2f})")
        
        # 计算每个角点相对于质心的角度
        import math
        angles = []
        for point in qr['points']:
            x, y = point
            angle = math.atan2(y - center_y, x - center_x)
            angles.append(math.degrees(angle))
        
        print("  角点角度:")
        for i, angle in enumerate(angles):
            print(f"    角点 {i+1}: {angle:.1f}°")
        
    else:
        print("❌ 未能检测到QR码或检测失败")
        print(f"响应: {data}")

if __name__ == "__main__":
    test_corner_points()
