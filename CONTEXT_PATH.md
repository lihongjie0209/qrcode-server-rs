# QR Code Server Environment Configuration

## 环境变量配置说明

QR码检测服务器现在支持通过环境变量配置多个方面，包括端口、上下文路径和对象池设置。

### 环境变量

- `PORT`: 设置服务器监听端口，默认为 `3000`
- `CONTEXT_PATH`: 设置服务器的上下文路径，默认为 `/`
- `POOL_INITIAL_SIZE`: 设置对象池初始大小，默认为 `10`，范围 1-100
- `POOL_MAX_SIZE`: 设置对象池最大大小，默认为 `50`，范围 初始大小-200
- `RUST_LOG`: 日志级别，可选 `error`, `warn`, `info`, `debug`, `trace`

### 使用示例

#### 1. 默认配置

```bash
# 默认环境变量
PORT=3000
CONTEXT_PATH=/
POOL_INITIAL_SIZE=10
POOL_MAX_SIZE=50

# 访问URL
http://localhost:3000/
http://localhost:3000/health
http://localhost:3000/detect/file
http://localhost:3000/ws
```

#### 2. 自定义端口和上下文路径

```bash
# 环境变量
PORT=8080
CONTEXT_PATH=/qrcode

# 访问URL
http://localhost:8080/qrcode/
http://localhost:8080/qrcode/health
http://localhost:8080/qrcode/detect/file
http://localhost:8080/qrcode/ws

# 根路径会自动重定向到上下文路径
http://localhost:8080/ -> http://localhost:8080/qrcode/
```

#### 3. 高性能配置

```bash
# 环境变量 - 适用于高并发场景
PORT=3000
CONTEXT_PATH=/api/v1/qrcode
POOL_INITIAL_SIZE=20
POOL_MAX_SIZE=100

# 访问URL
http://localhost:3000/api/v1/qrcode/
http://localhost:3000/api/v1/qrcode/health
http://localhost:3000/api/v1/qrcode/detect/file
http://localhost:3000/api/v1/qrcode/ws
```

### Docker 运行

#### 使用环境变量

```bash
# 默认配置
docker run -p 3000:3000 qrcode-server:latest

# 自定义端口
docker run -p 8080:8080 -e PORT=8080 qrcode-server:latest

# 自定义路径和对象池
docker run -p 3000:3000 \
  -e CONTEXT_PATH=/qrcode \
  -e POOL_INITIAL_SIZE=20 \
  -e POOL_MAX_SIZE=100 \
  qrcode-server:latest

# 完整配置示例
docker run -p 8080:8080 \
  -e PORT=8080 \
  -e CONTEXT_PATH=/api/v1/qrcode \
  -e POOL_INITIAL_SIZE=15 \
  -e POOL_MAX_SIZE=80 \
  -e RUST_LOG=debug \
  qrcode-server:latest
```

#### 使用 Docker Compose

修改 `docker-compose.yml` 文件中的环境变量：

```yaml
services:
  qrcode-server:
    environment:
      - CONTEXT_PATH=/qrcode  # 修改为所需的上下文路径
```

然后运行：

```bash
docker-compose up -d
```

### 对象池配置详解

#### 配置原则

1. **初始大小 (POOL_INITIAL_SIZE)**：
   - 服务器启动时创建的QR码检测器实例数量
   - 较大的值可以减少冷启动延迟
   - 但会增加内存使用量
   - 建议范围：5-50

2. **最大大小 (POOL_MAX_SIZE)**：
   - 对象池可以容纳的最大实例数量
   - 在高并发时动态创建新实例直到达到上限
   - 超过上限时请求会等待可用实例
   - 建议范围：20-200

#### 性能调优建议

```bash
# 低并发场景 (< 10 QPS)
POOL_INITIAL_SIZE=5
POOL_MAX_SIZE=20

# 中等并发场景 (10-50 QPS)
POOL_INITIAL_SIZE=10
POOL_MAX_SIZE=50

# 高并发场景 (50-100 QPS)
POOL_INITIAL_SIZE=20
POOL_MAX_SIZE=100

# 极高并发场景 (> 100 QPS)
POOL_INITIAL_SIZE=30
POOL_MAX_SIZE=150
```

### Nginx 反向代理配置

当使用自定义上下文路径时，需要相应调整 Nginx 配置：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    # 如果使用 /qrcode 作为上下文路径
    location /qrcode/ {
        proxy_pass http://qrcode_backend/qrcode/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # WebSocket 支持
    location /qrcode/ws {
        proxy_pass http://qrcode_backend/qrcode/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        # ... 其他设置
    }
}
```

### API 路径规范

无论使用什么上下文路径，API 端点的相对路径保持不变：

- `{CONTEXT_PATH}/` - 前端页面
- `{CONTEXT_PATH}/health` - 健康检查
- `{CONTEXT_PATH}/detect/file` - 文件上传检测
- `{CONTEXT_PATH}/detect/base64` - Base64 图片检测
- `{CONTEXT_PATH}/ws` - WebSocket 连接

### 前端自适应

前端页面会自动检测当前的上下文路径，无需手动配置。JavaScript 代码会：

1. 从 `window.location.pathname` 获取当前路径
2. 自动构建正确的 API 基础路径
3. 确保所有 API 调用使用正确的路径

### 注意事项

1. 上下文路径必须以 `/` 开头
2. 上下文路径不能以 `/` 结尾（除非是根路径 `/`）
3. 健康检查会尝试使用配置的上下文路径，如果失败会回退到根路径
4. 根路径访问会自动重定向到配置的上下文路径（除非上下文路径就是根路径）

### 测试验证

```bash
# 设置环境变量
export CONTEXT_PATH=/qrcode

# 启动服务器
./target/release/qrcode-server-rs

# 测试访问
curl http://localhost:3000/qrcode/health
curl http://localhost:3000/  # 应该重定向到 /qrcode/
```
