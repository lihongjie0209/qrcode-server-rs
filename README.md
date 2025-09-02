# QR Code Detection Server

🚀 基于 Rust 和 OpenCV WeChat QRCode 的高性能二维码检测服务器，支持 HTTP API 和 WebSocket 实时检测。

![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![OpenCV](https://img.shields.io/badge/opencv-4.6.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Performance](https://img.shields.io/badge/performance-80+_FPS-brightgreen.svg)

## ✨ 功能特性

### 🔥 核心功能
- ✅ **高精度检测**: 使用 OpenCV WeChat QRCode 算法
- ✅ **双重接口**: HTTP REST API + WebSocket 实时通信
- ✅ **多种输入**: 文件上传、Base64 编码、WebSocket 流
- ✅ **高性能**: 对象池架构，支持 80+ FPS 并发处理
- ✅ **实时统计**: 详细的性能指标和监控数据

### 🛠️ 技术优势
- ✅ **异步架构**: 基于 Tokio 异步运行时
- ✅ **内存优化**: 智能对象池管理 QR 检测器实例
- ✅ **可配置**: 丰富的环境变量配置选项
- ✅ **容器化**: 完整的 Docker 支持
- ✅ **Web界面**: 现代化的前端测试界面

### 🌐 部署特性
- ✅ **灵活路径**: 可配置 HTTP 上下文路径
- ✅ **端口配置**: 支持自定义端口
- ✅ **健康检查**: 完整的服务监控端点
- ✅ **反向代理**: 提供 Nginx 配置示例

## 🚀 快速开始

### 方式一：直接运行

```bash
# 克隆仓库
git clone https://github.com/your-username/qrcode-server-rs.git
cd qrcode-server-rs

# 构建项目
cargo build --release

# 启动服务器
./target/release/qrcode-server-rs
```

### 方式二：Docker 运行

```bash
# 构建镜像
docker build -t qrcode-server .

# 运行容器
docker run -p 3000:3000 qrcode-server
```

### 方式三：Docker Compose

```bash
# 启动完整服务
docker-compose up -d
```

## 🔧 环境变量配置

| 变量名 | 默认值 | 说明 | 示例 |
|--------|--------|------|------|
| `PORT` | `3000` | 服务器端口 | `8080` |
| `CONTEXT_PATH` | `/` | HTTP 上下文路径 | `/api/qrcode` |
| `POOL_INITIAL_SIZE` | `10` | 对象池初始大小 (1-100) | `20` |
| `POOL_MAX_SIZE` | `50` | 对象池最大大小 | `100` |
| `RUST_LOG` | `info` | 日志级别 | `debug` |

### 配置示例

```bash
# 基础配置
PORT=3000 CONTEXT_PATH=/ ./qrcode-server-rs

# 高性能配置
PORT=8080 \
CONTEXT_PATH=/api/qrcode \
POOL_INITIAL_SIZE=20 \
POOL_MAX_SIZE=100 \
RUST_LOG=info \
./qrcode-server-rs

# Docker 运行
docker run -p 8080:8080 \
  -e PORT=8080 \
  -e CONTEXT_PATH=/qrcode \
  -e POOL_INITIAL_SIZE=15 \
  -e POOL_MAX_SIZE=80 \
  qrcode-server
```

## 📡 API 接口

### HTTP REST API

#### 1. 健康检查
```bash
GET /{CONTEXT_PATH}/health?verbose=true
```

#### 2. 文件上传检测
```bash
POST /{CONTEXT_PATH}/detect/file
Content-Type: multipart/form-data
```

#### 3. Base64 检测
```bash
POST /{CONTEXT_PATH}/detect/base64
Content-Type: application/json

{
  "image": "base64_encoded_image_data"
}
```

### WebSocket API

#### 连接端点
```
ws://localhost:3000/{CONTEXT_PATH}/ws
```

#### 消息格式
```javascript
// 发送检测请求
{
  "type": "detect",
  "image": "base64_encoded_image"
}

// 关闭连接
{
  "type": "close"
}
```

### 检测结果格式

```json
{
  "success": true,
  "message": "Detected 1 QR code(s)",
  "count": 1,
  "qrcodes": [
    {
      "text": "https://www.example.com",
      "points": [[10.7, 13.0], [90.2, 15.1], [88.9, 95.3], [8.4, 93.2]],
      "bbox": {
        "x": 8.4,
        "y": 13.0,
        "width": 81.8,
        "height": 82.3
      }
    }
  ],
  "statistics": {
    "image_decode_time_ms": 2.55,
    "detection_time_ms": 18.64,
    "total_time_ms": 21.23,
    "image_width": 200,
    "image_height": 200,
    "pool_acquisition_time_ms": 0.05
  }
}
```

## 📊 性能基准

### WebSocket 性能
- **持续 FPS**: ~81 FPS
- **突发吞吐量**: 10,000+ 请求/秒
- **检测准确率**: 100%
- **平均响应时间**: ~12ms

### HTTP API 性能
- **并发处理**: 143+ QPS
- **图片解码**: 0.08-2.5ms
- **QR码检测**: 9-18ms
- **零失败率**: 100% 成功率

### 性能调优建议

```bash
# 低并发场景 (< 10 QPS)
POOL_INITIAL_SIZE=5 POOL_MAX_SIZE=20

# 中等并发场景 (10-50 QPS)
POOL_INITIAL_SIZE=10 POOL_MAX_SIZE=50

# 高并发场景 (50-100 QPS)
POOL_INITIAL_SIZE=20 POOL_MAX_SIZE=100

# 极高并发场景 (> 100 QPS)
POOL_INITIAL_SIZE=30 POOL_MAX_SIZE=150
```

## 🧪 测试工具

项目提供了完整的测试套件，位于 `test/` 目录：

```bash
cd test/

# 基础 WebSocket 测试
python3 simple_ws_test.py

# 快速性能测试
python3 quick_ws_test.py

# 完整性能测试
python3 websocket_fps_test.py

# HTTP API 基准测试
python3 benchmark.py

# 环境变量配置测试
python3 test_env_vars.py
```

## 🌐 Web 界面

访问 `http://localhost:3000/{CONTEXT_PATH}/` 使用现代化的 Web 界面：

- 📁 **文件上传**: 拖拽或点击上传图片
- 📝 **Base64 输入**: 直接输入 Base64 编码
- 📊 **性能统计**: 实时显示检测性能
- 🎯 **可视化**: QR码位置标记和边界框

## 🏗️ 项目结构

```
qrcode-server-rs/
├── src/
│   └── main.rs              # 主服务器代码
├── static/
│   └── index.html           # Web 前端界面
├── test/                    # 测试套件
│   ├── simple_ws_test.py    # 基础 WebSocket 测试
│   ├── quick_ws_test.py     # 快速性能测试
│   ├── websocket_fps_test.py # 完整性能测试
│   ├── benchmark.py         # HTTP API 基准测试
│   └── test_*.png          # 测试图片
├── models/                  # WeChat QRCode 模型文件
│   ├── detect.prototxt
│   ├── detect.caffemodel
│   ├── sr.prototxt
│   └── sr.caffemodel
├── Dockerfile              # Docker 构建文件
├── docker-compose.yml      # Docker Compose 配置
├── nginx.conf              # Nginx 反向代理配置
├── build.sh                # 自动化构建脚本
├── CONTEXT_PATH.md         # 详细配置文档
└── README.md               # 项目说明
```

## 🔧 技术栈

| 组件 | 版本 | 用途 |
|------|------|------|
| **Rust** | 1.75+ | 高性能系统编程语言 |
| **OpenCV** | 4.6.0 | 计算机视觉库 |
| **WeChat QRCode** | Latest | 高精度 QR 码检测算法 |
| **Axum** | 0.7 | 现代异步 Web 框架 |
| **Tokio** | Latest | 异步运行时 |
| **Object Pool** | 0.5.4 | 对象池管理 |
| **WebSocket** | Built-in | 实时通信支持 |

## 📝 使用示例

### 命令行测试

```bash
# 健康检查
curl http://localhost:3000/health

# 文件上传检测
curl -X POST -F "file=@test_qr.png" \
     http://localhost:3000/detect/file

# Base64 检测
curl -X POST -H "Content-Type: application/json" \
     -d '{"image":"iVBORw0KGgo..."}' \
     http://localhost:3000/detect/base64
```

### WebSocket 客户端示例

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
    // 发送图片检测
    ws.send(JSON.stringify({
        type: 'detect',
        image: 'base64_encoded_image'
    }));
};

ws.onmessage = (event) => {
    const result = JSON.parse(event.data);
    console.log('检测结果:', result);
};

// 关闭连接
ws.send(JSON.stringify({type: 'close'}));
```

## 🐳 Docker 部署

### 单容器部署

```bash
# 构建镜像
./build.sh

# 运行容器
docker run -d \
  --name qrcode-server \
  -p 3000:3000 \
  -e POOL_INITIAL_SIZE=20 \
  qrcode-server:latest
```

### 使用 Nginx 反向代理

```bash
# 启动完整服务栈
docker-compose up -d

# 通过 Nginx 访问
curl http://localhost/qrcode/health
```

## 📈 监控和日志

### 服务监控

```bash
# 查看服务状态
curl http://localhost:3000/health?verbose=true

# 检查 tmux 会话
tmux list-sessions
tmux attach -t qrcode-server

# 查看日志
tmux capture-pane -t qrcode-server -p | tail -20
```

### 性能监控

服务器提供详细的性能指标：

- **对象池状态**: 初始/最大实例数
- **请求统计**: 处理时间分布
- **系统信息**: OpenCV 版本、特性支持
- **实时指标**: QPS、响应时间、成功率

## 🤝 贡献指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📄 许可证

本项目基于 MIT 许可证开源 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [OpenCV](https://opencv.org/) - 计算机视觉库
- [WeChat QRCode](https://github.com/opencv/opencv_contrib/tree/master/modules/wechat_qrcode) - 高精度 QR 码检测算法
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Axum](https://github.com/tokio-rs/axum) - Web 框架

---

⭐ 如果这个项目对您有帮助，请给个 Star！
