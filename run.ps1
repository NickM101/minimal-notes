# Load cargo and cmake into the current session PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\tools\cmake\bin;C:\Users\nickm\tools\cmake\bin;$env:PATH"

# Kill any lingering instance to prevent Windows file lock error 5
Get-Process -Name minimal-notes -ErrorAction SilentlyContinue | Stop-Process -Force

Write-Host "Launching Minimal Notes..." -ForegroundColor Cyan
cargo run
