@echo off
title Launch UloOS 1.2 Web Simulator
echo ===========================================================
echo         LAUNCHING ULOOS 1.2 WEB SIMULATOR DEV SERVER       
echo ===========================================================
echo [System] Booting local Node.js development server...
cd /d "%~dp0"
:: Launch npm run dev in a new separate command prompt so it runs in the background
start "UloOS Web Server" cmd /k "npm run dev"
echo [System] Waiting 3 seconds for dev server to initialize...
timeout /t 3 /nobreak >nul
echo [System] Launching your web browser to local simulator...
start http://localhost:3000
echo ===========================================================
echo [Success] Web simulator launched! Keep the server window open.
echo ===========================================================
timeout /t 5 >nul
