#Requires -Version 5.1
<#
.SYNOPSIS
    Creates a Windows Start Menu shortcut for VibeFlow
    Made by DerJannik

.DESCRIPTION
    Adds VibeFlow to the Windows Start Menu so you can search and launch it easily.

.EXAMPLE
    .\install-shortcut.ps1
#>

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "🔗 Creating VibeFlow Start Menu Shortcut..." -ForegroundColor Cyan
Write-Host ""

# Determine the executable path
$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $projectRoot) {
    $projectRoot = (Get-Location).Path
}

$exePath = Join-Path $projectRoot "src-tauri\target\release\vibeflow.exe"
$devExePath = Join-Path $projectRoot "src-tauri\target\debug\vibeflow.exe"

# Check which exe exists
if (Test-Path $exePath) {
    $targetPath = $exePath
    Write-Host "  Using release build: $exePath" -ForegroundColor Green
}
elseif (Test-Path $devExePath) {
    $targetPath = $devExePath
    Write-Host "  Using debug build: $devExePath" -ForegroundColor Yellow
}
else {
    Write-Host "  ❌ VibeFlow executable not found!" -ForegroundColor Red
    Write-Host "     Please build the app first with: npm run build" -ForegroundColor Yellow
    Write-Host "     Or run in dev mode: npm run dev" -ForegroundColor Yellow
    exit 1
}

# Create shortcut in Start Menu
$startMenuPath = [Environment]::GetFolderPath("StartMenu")
$shortcutPath = Join-Path $startMenuPath "Programs\VibeFlow.lnk"

$WshShell = New-Object -ComObject WScript.Shell
$shortcut = $WshShell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $targetPath
$shortcut.WorkingDirectory = $projectRoot
$shortcut.Description = "VibeFlow - Voice-to-Text by DerJannik"
$shortcut.IconLocation = $targetPath

# Check for icon file
$iconPath = Join-Path $projectRoot "src-tauri\icons\icon.ico"
if (Test-Path $iconPath) {
    $shortcut.IconLocation = $iconPath
}

$shortcut.Save()

Write-Host ""
Write-Host "  ✅ Shortcut created successfully!" -ForegroundColor Green
Write-Host "     Location: $shortcutPath" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  You can now search 'VibeFlow' in Windows Start Menu!" -ForegroundColor Cyan
Write-Host ""
