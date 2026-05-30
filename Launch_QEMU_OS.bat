@echo off
title Launch UloOS 1.2 in QEMU
echo ===========================================================
echo             LAUNCHING ULOOS 1.2 IN FULLSCREEN QEMU         
echo ===========================================================
echo [System] Booting pre-compiled UloOS bare-metal binary...

set "BIN_PATH=%~dp0uloos-kernel\target\x86_64-unknown-none\debug\bootimage-uloos-kernel.bin"
set "QEMU_PATH=C:\Program Files\qemu\qemu-system-x86_64.exe"

if exist "%BIN_PATH%" (
    echo [System] Cleaning stale binary to force fresh build...
    del /f /q "%BIN_PATH%" >nul 2>&1
)

echo [System] Rebuilding Rust kernel and booting QEMU...
cd /d "%~dp0uloos-kernel"
cargo run

if %errorlevel% neq 0 (
    echo.
    echo [Error] Failed to launch QEMU. Make sure QEMU is installed at C:\Program Files\qemu\
    pause
)
