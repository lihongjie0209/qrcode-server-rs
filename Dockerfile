# 多阶段构建 - 构建阶段
FROM rust:1.89 AS builder

# 设置环境变量以优化构建
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# 安装OpenCV和构建依赖
RUN apt-get update && apt-get install -y \
    libopencv-dev \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# 先复制依赖文件以利用Docker缓存
COPY Cargo.toml Cargo.lock ./

# 创建虚拟源文件来构建依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 复制实际源代码和静态文件
COPY src ./src
COPY static ./static

# 重新构建应用（只编译源代码变更）
RUN touch src/main.rs && cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libopencv-core-dev \
    libopencv-imgproc-dev \
    libopencv-imgcodecs-dev \
    libopencv-objdetect-dev \
    libopencv-dnn-dev \
    libopencv-contrib-dev \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# 创建非root用户
RUN groupadd -r qrcode && useradd -r -g qrcode -d /app -s /bin/bash qrcode

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/qrcode-server-rs ./

# 复制静态文件
COPY static ./static

# 创建并复制模型目录
RUN mkdir -p models
COPY models/ ./models/

# 创建测试目录（可选）
COPY test ./test

# 设置权限
RUN chown -R qrcode:qrcode /app

# 切换到非root用户
USER qrcode

# 暴露端口
EXPOSE 3000

# 设置环境变量
ENV RUST_LOG=info
ENV PORT=3000
ENV CONTEXT_PATH=/
ENV POOL_INITIAL_SIZE=10
ENV POOL_MAX_SIZE=50

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:3000${CONTEXT_PATH}/health || curl -f http://localhost:3000/health || exit 1

# 设置标签
LABEL org.opencontainers.image.title="QR Code Detection Server" \
      org.opencontainers.image.description="High-performance QR code detection server with WebSocket support" \
      org.opencontainers.image.version="1.0.0" \
      org.opencontainers.image.authors="QR Code Server Team"

# 启动应用
CMD ["./qrcode-server-rs"]
