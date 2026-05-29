@echo off
title Launch UloOS 1.2 in QEMU
echo ===========================================================
echo             LAUNCHING ULOOS 1.2 IN FULLSCREEN QEMU         
echo ===========================================================
echo [System] Booting bare-metal Rust kernel...
cd /d "%~dp0uloos-kernel"
cargo run
if %errorlevel% neq 0 (
    echo.
    echo [Error] Failed to run QEMU. Make sure QEMU is installed at C:\Program Files\qemu\
    pause
)
