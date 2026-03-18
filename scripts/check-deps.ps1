#Requires -Version 5.1
<#
.SYNOPSIS
    VibeFlow Build Dependency Checker
    Made by DerJannik

.DESCRIPTION
    Quickly verifies all required build dependencies are installed.
    Run this before 'npm run dev' to catch missing dependencies early.

.EXAMPLE
    .\check-deps.ps1
#>

$ErrorActionPreference = "Continue"

function Write-Check { 
    param($name, $found, $path)
    if ($found) {
        Write-Host "  ✅ $name" -ForegroundColor Green
        if ($path) { Write-Host "     $path" -ForegroundColor DarkGray }
    }
    else {
        Write-Host "  ❌ $name - NOT FOUND" -ForegroundColor Red
        $script:missing++
    }
}

$missing = 0

Write-Host ""
Write-Host "🔍 VibeFlow Dependency Check" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor DarkGray
Write-Host ""

# 1. Node.js
$node = Get-Command node -ErrorAction SilentlyContinue
Write-Check "Node.js" $node $node.Source

# 2. Rust/Cargo
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
Write-Check "Rust (cargo)" $cargo $cargo.Source

# 3. MSVC link.exe
$linkPaths = @(
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe",
    "C:\Program Files\Microsoft Visual Studio\2022\*\VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe"
)
$link = $linkPaths | ForEach-Object { Get-Item $_ -ErrorAction SilentlyContinue } | Select-Object -First 1
Write-Check "MSVC Linker (link.exe)" $link $link.FullName

# 4. Windows SDK
$sdkPath = "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.*\um\x64\kernel32.lib"
$sdk = Get-Item $sdkPath -ErrorAction SilentlyContinue | Select-Object -First 1
Write-Check "Windows SDK (kernel32.lib)" $sdk $sdk.FullName

# 5. LLVM/libclang
$clangPaths = @(
    "C:\Program Files\LLVM\bin\libclang.dll",
    "C:\Program Files (x86)\LLVM\bin\libclang.dll"
)
$clang = $clangPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
Write-Check "LLVM (libclang.dll)" $clang $clang

# 6. CMake
$cmake = Get-Command cmake -ErrorAction SilentlyContinue
Write-Check "CMake" $cmake $cmake.Source

Write-Host ""
Write-Host "═══════════════════════════════════════" -ForegroundColor DarkGray

if ($missing -eq 0) {
    Write-Host "✅ All dependencies found! Ready to build." -ForegroundColor Green
    Write-Host ""
    Write-Host "Run: npm run dev" -ForegroundColor White
    exit 0
}
else {
    Write-Host "❌ Missing $missing dependencies." -ForegroundColor Red
    Write-Host ""
    Write-Host "Run: .\scripts\setup-windows.ps1 to install them." -ForegroundColor Yellow
    exit 1
}
