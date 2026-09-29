$ScriptContent = @'
$ErrorActionPreference = "Stop"

$InstallDir = "$env:LOCALAPPDATA\winfetch\bin"
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$ExeUrl = "https://github.com/click-for-games/winfetch/releases/latest/download/winfetch.exe"
Write-Host "Downloading winfetch..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $ExeUrl -OutFile "$InstallDir\winfetch.exe"

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to PATH." -ForegroundColor Green
}

# Add to current session PATH immediately
if ($env:PATH -notlike "*$InstallDir*") {
    $env:PATH += ";$InstallDir"
}

Write-Host "`nwinfetch installed successfully!" -ForegroundColor Green
Write-Host "Type 'winfetch' to launch." -ForegroundColor Yellow
'@

[System.IO.File]::WriteAllText("$PWD\install.ps1", $ScriptContent, (New-Object System.Text.UTF8Encoding $false))