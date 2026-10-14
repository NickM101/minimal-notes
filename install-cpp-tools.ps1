Write-Host "Opening Visual Studio C++ Build Tools Installer..." -ForegroundColor Cyan
Write-Host "Make sure 'Desktop development with C++' is selected, then click Install." -ForegroundColor Yellow

Start-Process -FilePath "$env:TEMP\vs_buildtools.exe" -ArgumentList "--passive --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
