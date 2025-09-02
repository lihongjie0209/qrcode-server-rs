#!/bin/bash

echo "=== 构建Docker镜像 ==="
docker build --no-cache -t qrcode-server-rs:debug .

echo "=== 检查二进制文件依赖 ==="
docker run --rm qrcode-server-rs:debug ldd ./qrcode-server-rs

echo "=== 检查模型文件 ==="
docker run --rm qrcode-server-rs:debug ls -la models/

echo "=== 检查OpenCV库文件 ==="
docker run --rm qrcode-server-rs:debug find /usr/lib -name "*opencv*" | head -10

echo "=== 启动容器（详细日志） ==="
docker run --rm -p 3000:3000 -e RUST_LOG=debug -e RUST_BACKTRACE=full qrcode-server-rs:debug
