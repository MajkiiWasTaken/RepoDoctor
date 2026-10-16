
<#
File: uninstall.ps1
Author: Michal Švrček
RepoDoctor Windows uninstaller.
ver. 0.6.0
#>

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\RepoDoctor"
$Binary = Join-Path $InstallDir "rdoctor.exe"

if (Test-Path $Binary) {
    Remove-Item -LiteralPath $Binary -Force
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

$Entries = @(
    $UserPath -split ';' | Where-Object {
        $_.Trim() -ne "" -and
        $_.TrimEnd('\') -ine $InstallDir.TrimEnd('\')
    }
)

[Environment]::SetEnvironmentVariable(
    "Path",
    ($Entries -join ';'),
    "User"
)

Write-Host "[ OK ] RepoDoctor uninstalled." -ForegroundColor Green
Write-Host "Project configuration files were preserved."
