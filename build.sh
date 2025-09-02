#!/bin/bash

# QR Code Server Docker Build Script
# 二维码服务器Docker构建脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
IMAGE_NAME="qrcode-server"
TAG="${1:-latest}"
FULL_IMAGE_NAME="${IMAGE_NAME}:${TAG}"

echo -e "${BLUE}🚀 Building QR Code Detection Server Docker Image${NC}"
echo -e "${BLUE}=================================================${NC}"

# 检查Docker是否安装
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# 检查模型文件是否存在
if [ ! -d "models" ] || [ -z "$(ls -A models)" ]; then
    echo -e "${YELLOW}⚠️  Warning: models directory is empty or missing${NC}"
    echo -e "${YELLOW}   Please ensure WeChat QRCode model files are in the models/ directory${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 检查静态文件
if [ ! -d "static" ]; then
    echo -e "${YELLOW}⚠️  Warning: static directory is missing${NC}"
fi

# 构建镜像
echo -e "${GREEN}🔨 Building Docker image: ${FULL_IMAGE_NAME}${NC}"
docker build -t "${FULL_IMAGE_NAME}" .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Docker image built successfully!${NC}"
    
    # 显示镜像信息
    echo -e "${BLUE}📊 Image Information:${NC}"
    docker images "${IMAGE_NAME}" --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.CreatedAt}}\t{{.Size}}"
    
    echo -e "${BLUE}💡 Usage examples:${NC}"
    echo -e "   ${GREEN}# Run with Docker:${NC}"
    echo -e "   docker run -p 3000:3000 ${FULL_IMAGE_NAME}"
    echo -e ""
    echo -e "   ${GREEN}# Run with Docker Compose:${NC}"
    echo -e "   docker-compose up -d"
    echo -e ""
    echo -e "   ${GREEN}# Push to registry:${NC}"
    echo -e "   docker tag ${FULL_IMAGE_NAME} your-registry.com/${FULL_IMAGE_NAME}"
    echo -e "   docker push your-registry.com/${FULL_IMAGE_NAME}"
    
else
    echo -e "${RED}❌ Docker build failed!${NC}"
    exit 1
fi
