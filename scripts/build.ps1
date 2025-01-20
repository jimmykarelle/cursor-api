# 参数处理
param(
    [switch]$Static,
    [switch]$Help,
    [ValidateSet("x86_64", "aarch64", "i686")]
    [string]$Architecture
)

# 设置错误时停止执行
$ErrorActionPreference = "Stop"

# 颜色输出函数
function Write-Info  { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Warn  { param($Message) Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red; exit 1 }

# 检查必要的工具
function Check-Requirements {
    $tools = @("cargo", "protoc", "npm", "node")
    $missing = @()

    foreach ($tool in $tools) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            $missing += $tool
        }
    }

    if ($missing.Count -gt 0) {
        Write-Error "缺少必要工具: $($missing -join ', ')"
    }
}

# 帮助信息
function Show-Help {
    Write-Host @"
用法: $(Split-Path $MyInvocation.ScriptName -Leaf) [选项]

选项:
  -Static        使用静态链接（默认动态链接）
  -Help          显示此帮助信息

不带参数时使用默认配置构建
"@
}

# 构建函数
function Build-Target {
    param (
        [string]$Target,
        [string]$RustFlags
    )

    Write-Info "正在构建 $Target..."

    # 设置环境变量
    $env:RUSTFLAGS = $RustFlags

    # 构建
    if ($Target -ne (rustc -Vv | Select-String "host: (.*)" | ForEach-Object { $_.Matches.Groups[1].Value })) {
        cargo build --target $Target --release
    } else {
        cargo build --release
    }

    # 移动编译产物到 release 目录
    $binaryName = "cursor-api"
    if ($Static) {
        $binaryName += "-static"
    }

    $binaryPath = if ($Target -eq (rustc -Vv | Select-String "host: (.*)" | ForEach-Object { $_.Matches.Groups[1].Value })) {
        "target/release/cursor-api.exe"
    } else {
        "target/$Target/release/cursor-api.exe"
    }

    if (Test-Path $binaryPath) {
        Copy-Item $binaryPath "release/$binaryName-$Target.exe"
        Write-Info "完成构建 $Target"
    } else {
        Write-Warn "构建产物未找到: $Target"
        Write-Warn "查找路径: $binaryPath"
        Write-Warn "当前目录内容:"
        Get-ChildItem -Recurse target/
        return $false
    }

    return $true
}

if ($Help) {
    Show-Help
    exit 0
}

# 检查依赖
Check-Requirements

# 创建 release 目录
New-Item -ItemType Directory -Force -Path release | Out-Null

# 设置静态链接标志
$rustFlags = ""
if ($Static) {
    $rustFlags = "-C target-feature=+crt-static"
}

# 获取目标架构
$arch = if ($Architecture) {
    $Architecture
} else {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { "x86_64" }
        "ARM64" { "aarch64" }
        "X86" { "i686" }
        default { Write-Error "不支持的架构: $env:PROCESSOR_ARCHITECTURE" }
    }
}
$target = "$arch-pc-windows-msvc"

Write-Info "开始构建..."
if (-not (Build-Target -Target $target -RustFlags $rustFlags)) {
    Write-Error "构建失败"
}

Write-Info "构建完成！"