@echo off
set "PATH=%USERPROFILE%\.cargo\bin;C:\Users\nickm\tools\cmake\bin;%PATH%"
echo Launching Minimal Notes...
start "" "%~dp0target\debug\minimal-notes.exe"
