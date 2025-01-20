#!/bin/bash

# 设置错误时退出
set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}"
    exit 1
}

# 检查是否为 root 用户（FreeBSD 和 Linux）
if [ "$(uname)" != "Darwin" ] && [ "$EUID" -ne 0 ]; then
    error "请使用 root 权限运行此脚本 (sudo ./setup.sh)"
fi

# 检测包管理器
if command -v brew &> /dev/null; then
    PKG_MANAGER="brew"
    info "检测到 macOS/Homebrew 系统"
elif command -v pkg &> /dev/null; then
    PKG_MANAGER="pkg"
    info "检测到 FreeBSD 系统"
elif command -v apt-get &> /dev/null; then
    PKG_MANAGER="apt-get"
    info "检测到 Debian/Ubuntu 系统"
elif command -v dnf &> /dev/null; then
    PKG_MANAGER="dnf"
    info "检测到 Fedora/RHEL 系统"
elif command -v yum &> /dev/null; then
    PKG_MANAGER="yum"
    info "检测到 CentOS 系统"
else
    error "未检测到支持的包管理器"
fi

# 更新包管理器缓存
info "更新包管理器缓存..."
case $PKG_MANAGER in
    "brew")
        brew update
        ;;
    "pkg")
        pkg update
        ;;
    *)
        $PKG_MANAGER update -y
        ;;
esac

# 安装基础构建工具
info "安装基础构建工具..."
case $PKG_MANAGER in
    "brew")
        brew install \
            protobuf \
            pkg-config \
            openssl \
            curl \
            git \
            node
        ;;
    "pkg")
        pkg install -y \
            gmake \
            protobuf \
            pkgconf \
            openssl \
            curl \
            git \
            node
        ;;
    "apt-get")
        $PKG_MANAGER install -y --no-install-recommends \
            build-essential \
            protobuf-compiler \
            pkg-config \
            libssl-dev \
            ca-certificates \
            curl \
            tzdata \
            git
        ;;
    *)
        $PKG_MANAGER install -y \
            gcc \
            gcc-c++ \
            make \
            protobuf-compiler \
            pkg-config \
            openssl-devel \
            ca-certificates \
            curl \
            tzdata \
            git
        ;;
esac

# 安装 Node.js 和 npm（如果还没有通过包管理器安装）
if ! command -v node &> /dev/null && [ "$PKG_MANAGER" != "brew" ] && [ "$PKG_MANAGER" != "pkg" ]; then
    info "安装 Node.js 和 npm..."
    if [ "$PKG_MANAGER" = "apt-get" ]; then
        curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
        $PKG_MANAGER install -y nodejs
    else
        curl -fsSL https://rpm.nodesource.com/setup_lts.x | bash -
        $PKG_MANAGER install -y nodejs
    fi
fi

# 安装 Rust（如果未安装）
if ! command -v rustc &> /dev/null; then
    info "安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
fi

# 添加目标平台
info "添加 Rust 目标平台..."
case "$(uname)" in
    "FreeBSD")
        rustup target add x86_64-unknown-freebsd
        ;;
    "Darwin")
        rustup target add x86_64-apple-darwin aarch64-apple-darwin
        ;;
    *)
        rustup target add x86_64-unknown-linux-gnu
        ;;
esac

# 清理包管理器缓存
case $PKG_MANAGER in
    "apt-get")
        rm -rf /var/lib/apt/lists/*
        ;;
    "pkg")
        pkg clean -y
        ;;
esac

# 设置时区（除了 macOS）
if [ "$(uname)" != "Darwin" ]; then
    info "设置时区为 Asia/Shanghai..."
    ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
fi

echo -e "${GREEN}安装完成！${NC}"