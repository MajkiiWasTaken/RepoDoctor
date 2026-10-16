
<#
File: install.ps1
Author: Michal Švrček
RepoDoctor Windows installer.
ver. 0.6.0
#>

param(
    [string]$BinaryPath
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

if (-not $BinaryPath) {
    $PackagedBinary = Join-Path $ScriptDir "rdoctor.exe"
    $BuildBinary = Join-Path $ProjectRoot "target\release\rdoctor.exe"

    if (Test-Path $PackagedBinary) {
        $BinaryPath = $PackagedBinary
    }
    else {
        $BinaryPath = $BuildBinary
    }
}

if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "rdoctor.exe not found: $BinaryPath"
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\RepoDoctor"
$Destination = Join-Path $InstallDir "rdoctor.exe"

Write-Host "RepoDoctor Installer" -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$Source = (Resolve-Path -LiteralPath $BinaryPath).Path

if ($Source -ine $Destination) {
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

$Entries = @(
    $UserPath -split ';' | Where-Object {
        $_.Trim() -ne ""
    }
)

if (-not ($Entries | Where-Object {
    $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')
})) {
    $Entries += $InstallDir

    [Environment]::SetEnvironmentVariable(
        "Path",
        ($Entries -join ';'),
        "User"
    )
}

# Make the command available in this PowerShell session.
if (-not (($env:Path -split ';') | Where-Object {
    $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')
})) {
    $env:Path += ";$InstallDir"
}

Write-Host "[ OK ] Installed: $Destination" -ForegroundColor Green
& $Destination --version

Write-Host "[ OK ] Run: rdoctor --help" -ForegroundColor Green
