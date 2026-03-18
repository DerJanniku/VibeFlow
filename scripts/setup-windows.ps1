#Requires -Version 5.1
<#
.SYNOPSIS
    VibeFlow Windows Development Environment Setup
    Made by DerJannik | https://de.fiverr.com/s/xXgY29x

.DESCRIPTION
    Automatically installs all required dependencies for building VibeFlow from source on Windows:
    - Visual Studio Build Tools (C++ workload)
    - Windows SDK
    - LLVM (for libclang/bindgen)
    - CMake (for whisper.cpp)

.EXAMPLE
    .\setup-windows.ps1
#>

param(
    [switch]$SkipConfirm,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# Colors
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Cyan }
function Write-Warn { param($msg) Write-Host "⚠️  $msg" -ForegroundColor Yellow }
function Write-Err { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Magenta
Write-Host "║          VibeFlow - Windows Development Setup                ║" -ForegroundColor Magenta
Write-Host "║                    by DerJannik                              ║" -ForegroundColor Magenta
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Magenta
Write-Host ""

# Check if running as admin (recommended but not required)
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warn "Not running as Administrator. Some installations may require elevation."
}

# Check for winget
Write-Info "Checking for winget..."
$winget = Get-Command winget -ErrorAction SilentlyContinue
if (-not $winget) {
    Write-Err "winget is not installed. Please install App Installer from the Microsoft Store."
    exit 1
}
Write-Success "winget found"

# Dependencies to install
$dependencies = @(
    @{
        Name = "Visual Studio Build Tools"
        WingetId = "Microsoft.VisualStudio.2022.BuildTools"
        Check = { Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" }
        Override = "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"
    },
    @{
        Name = "Windows SDK"
        WingetId = "Microsoft.WindowsSDK.10.0.22621"
        Check = { Test-Path "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0" }
        Override = $null
    },
    @{
        Name = "LLVM (libclang)"
        WingetId = "LLVM.LLVM"
        Check = { Test-Path "C:\Program Files\LLVM\bin\libclang.dll" }
        Override = $null
    },
    @{
        Name = "CMake"
        WingetId = "Kitware.CMake"
        Check = { Get-Command cmake -ErrorAction SilentlyContinue }
        Override = $null
    }
)

# Check existing installations
Write-Info "Checking existing installations..."
$toInstall = @()
foreach ($dep in $dependencies) {
    if (& $dep.Check) {
        Write-Success "$($dep.Name) is already installed"
    } else {
        Write-Warn "$($dep.Name) is NOT installed"
        $toInstall += $dep
    }
}

if ($toInstall.Count -eq 0) {
    Write-Host ""
    Write-Success "All dependencies are already installed! You're ready to build VibeFlow."
    Write-Host ""
    Write-Info "Run: npm run dev"
    exit 0
}

# Confirm installation
Write-Host ""
Write-Info "The following dependencies will be installed:"
foreach ($dep in $toInstall) {
    Write-Host "  - $($dep.Name)" -ForegroundColor White
}
Write-Host ""

if (-not $SkipConfirm) {
    $confirm = Read-Host "Continue? (Y/n)"
    if ($confirm -eq "n" -or $confirm -eq "N") {
        Write-Info "Installation cancelled."
        exit 0
    }
}

# Install dependencies
$failed = @()
foreach ($dep in $toInstall) {
    Write-Host ""
    Write-Info "Installing $($dep.Name)..."
    
    $args = @("install", "--id", $dep.WingetId, "--silent", "--accept-package-agreements", "--accept-source-agreements")
    if ($dep.Override) {
        $args += @("--override", $dep.Override)
    }
    
    try {
        $process = Start-Process -FilePath "winget" -ArgumentList $args -Wait -PassThru -NoNewWindow
        if ($process.ExitCode -eq 0) {
            Write-Success "$($dep.Name) installed successfully"
        } else {
            Write-Err "$($dep.Name) installation failed (exit code: $($process.ExitCode))"
            $failed += $dep.Name
        }
    } catch {
        Write-Err "Failed to install $($dep.Name): $_"
        $failed += $dep.Name
    }
}

# Summary
Write-Host ""
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Magenta

if ($failed.Count -eq 0) {
    Write-Success "All dependencies installed successfully!"
    Write-Host ""
    Write-Info "IMPORTANT: You may need to restart your terminal for PATH changes to take effect."
    Write-Host ""
    Write-Info "To build VibeFlow, run:"
    Write-Host "  npm run dev" -ForegroundColor White
} else {
    Write-Err "Some installations failed: $($failed -join ', ')"
    Write-Info "Please try installing them manually or run this script as Administrator."
}

Write-Host ""
