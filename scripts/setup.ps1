# 设置错误时停止执行
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"  # 加快下载速度

# 颜色输出函数
function Write-Info  { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Warn  { param($Message) Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-Success { param($Message) Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Error { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red; exit 1 }

# 检查管理员权限
function Test-Administrator {
    $user = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal $user
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Administrator)) {
    Write-Error "请以管理员权限运行此脚本"
}

# 帮助信息
function Show-Help {
    Write-Host @"
用法: $(Split-Path $MyInvocation.ScriptName -Leaf) [选项]

选项:
  -NoVS           不安装 Visual Studio Build Tools
  -NoRust         不安装 Rust
  -NoNode         不安装 Node.js
  -Help           显示此帮助信息

示例:
  .\setup.ps1
  .\setup.ps1 -NoVS
  .\setup.ps1 -NoRust -NoNode
"@
}

# 参数处理
param(
    [switch]$NoVS,
    [switch]$NoRust,
    [switch]$NoNode,
    [switch]$Help
)

if ($Help) {
    Show-Help
    exit 0
}

# 检查并安装 Chocolatey
function Install-Chocolatey {
    Write-Info "检查 Chocolatey..."
    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Info "安装 Chocolatey..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        try {
            Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
        }
        catch {
            Write-Error "安装 Chocolatey 失败: $_"
        }
        # 刷新环境变量
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    }
}

# 安装 Visual Studio Build Tools
function Install-VSBuildTools {
    if ($NoVS) {
        Write-Info "跳过 Visual Studio Build Tools 安装"
        return
    }

    Write-Info "检查 Visual Studio Build Tools..."
    $vsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vsPath)) {
        Write-Info "安装 Visual Studio Build Tools..."
        try {
            # 下载安装程序
            $vsInstallerUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
            $vsInstallerPath = "$env:TEMP\vs_BuildTools.exe"
            Invoke-WebRequest -Uri $vsInstallerUrl -OutFile $vsInstallerPath

            # 安装
            $process = Start-Process -FilePath $vsInstallerPath -ArgumentList `
                "--quiet", "--wait", "--norestart", "--nocache", `
                "--installPath", "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools", `
                "--add", "Microsoft.VisualStudio.Workload.VCTools" `
                -NoNewWindow -Wait -PassThru

            if ($process.ExitCode -ne 0) {
                Write-Error "Visual Studio Build Tools 安装失败"
            }

            Remove-Item $vsInstallerPath -Force
        }
        catch {
            Write-Error "安装 Visual Studio Build Tools 失败: $_"
        }
    }
    else {
        Write-Info "Visual Studio Build Tools 已安装"
    }
}

# 安装 Rust
function Install-Rust {
    if ($NoRust) {
        Write-Info "跳过 Rust 安装"
        return
    }

    Write-Info "检查 Rust..."
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Info "安装 Rust..."
        try {
            $rustupInit = "$env:TEMP\rustup-init.exe"
            Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $rustupInit
            Start-Process -FilePath $rustupInit -ArgumentList "-y" -Wait
            Remove-Item $rustupInit -Force

            # 刷新环境变量
            $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
        }
        catch {
            Write-Error "安装 Rust 失败: $_"
        }
    }

    # 添加目标平台
    Write-Info "配置 Rust 目标平台..."
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
    rustup target add "$arch-pc-windows-msvc"
}

# 安装其他工具
function Install-Tools {
    Write-Info "安装必要工具..."
    
    # 安装 protoc
    if (-not (Get-Command protoc -ErrorAction SilentlyContinue)) {
        Write-Info "安装 Protocol Buffers..."
        choco install -y protoc
    }

    # 安装 Git
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Info "安装 Git..."
        choco install -y git
    }

    # 安装 Node.js
    if (-not $NoNode -and -not (Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Info "安装 Node.js..."
        choco install -y nodejs
    }

    # 刷新环境变量
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

# 主流程
try {
    Write-Info "开始安装必要组件..."
    
    Install-Chocolatey
    Install-VSBuildTools
    Install-Rust
    Install-Tools

    Write-Success "安装完成！"
}
catch {
    Write-Error "安装过程中出现错误: $_"
}