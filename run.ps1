# Load cargo and cmake into the current session PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\tools\cmake\bin;C:\Users\nickm\tools\cmake\bin;$env:PATH"

Write-Host "Launching Minimal Notes..." -ForegroundColor Cyan
cargo run
