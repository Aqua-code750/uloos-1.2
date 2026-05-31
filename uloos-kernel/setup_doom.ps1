# ============================================
# UloOS DOOM Setup Script
# ============================================
# Downloads the doomgeneric source code and DOOM1.WAD shareware
# so you can build real DOOM into your bare-metal UloOS kernel.
#
# Usage: Right-click this file → "Run with PowerShell"
#   OR:  Open terminal in uloos-kernel/ and run: powershell -File setup_doom.ps1
#
# Prerequisites:
#   - Git (for cloning doomgeneric)
#   - A C compiler: clang (recommended) or gcc
#     Install via: winget install LLVM.LLVM
#     Or:          scoop install llvm
# ============================================

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  UloOS DOOM Setup - Bare Metal Edition" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Get the script's directory (should be uloos-kernel/)
$kernelDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $kernelDir

# ---- Step 1: Clone doomgeneric ----
Write-Host "[1/3] Cloning doomgeneric (DOOM source code)..." -ForegroundColor Yellow

if (Test-Path "doomgeneric") {
    Write-Host "  -> doomgeneric/ already exists. Pulling latest..." -ForegroundColor Gray
    Push-Location "doomgeneric"
    git pull --quiet 2>$null
    Pop-Location
} else {
    git clone https://github.com/ozkl/doomgeneric.git
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to clone doomgeneric. Make sure git is installed." -ForegroundColor Red
        exit 1
    }
}

Write-Host "  -> doomgeneric source ready!" -ForegroundColor Green

# ---- Step 2: Download DOOM1.WAD (Shareware) ----
Write-Host ""
Write-Host "[2/3] Downloading DOOM1.WAD (Shareware Edition)..." -ForegroundColor Yellow

$wadDir = "doomgeneric/doomgeneric"
$wadPath = "$wadDir/DOOM1.WAD"

if (Test-Path $wadPath) {
    $size = (Get-Item $wadPath).Length
    if ($size -gt 1000000) {
        Write-Host "  -> DOOM1.WAD already exists ($([math]::Round($size/1MB, 1)) MB)" -ForegroundColor Gray
    } else {
        Write-Host "  -> DOOM1.WAD exists but seems too small. Re-downloading..." -ForegroundColor Yellow
        Remove-Item $wadPath
    }
}

if (-not (Test-Path $wadPath)) {
    # Download DOOM1.WAD shareware from a reliable mirror
    $wadUrl = "https://distro.ibiblio.org/slitaz/sources/packages/d/doom1.wad"
    try {
        Invoke-WebRequest -Uri $wadUrl -OutFile $wadPath -UseBasicParsing
        $size = (Get-Item $wadPath).Length
        Write-Host "  -> Downloaded DOOM1.WAD ($([math]::Round($size/1MB, 1)) MB)" -ForegroundColor Green
    } catch {
        Write-Host "  -> Primary mirror failed. Trying alternative..." -ForegroundColor Yellow
        $wadUrl2 = "https://www.quaddicted.com/files/idgames/idstuff/doom/win95/doom95.zip"
        Write-Host "  -> Please manually download DOOM1.WAD and place it at:" -ForegroundColor Red
        Write-Host "     $wadPath" -ForegroundColor White
        Write-Host "  -> You can get it from: https://doomwiki.org/wiki/DOOM1.WAD" -ForegroundColor White
    }
}

# ---- Step 3: Create dummy.c (missing file stub) ----
Write-Host ""
Write-Host "[3/3] Creating build stubs..." -ForegroundColor Yellow

$dummyC = "$wadDir/dummy.c"
if (-not (Test-Path $dummyC)) {
    @"
// Dummy stubs for doomgeneric bare-metal build
// These functions are not needed on UloOS since the Rust kernel provides them.

#include <stdio.h>

// WAD file path — doomgeneric uses this to find the WAD
// On UloOS, the WAD is embedded in the kernel binary
const char* D_DoomExeDir(void) { return ""; }

// Stub for mkdir (not needed on bare metal)
int M_MakeDirectory(const char* path) { return 0; }

// Stub for file existence check
int M_FileExists(const char* filename) {
    // Only the embedded WAD "exists"
    return 0;
}
"@ | Out-File -FilePath $dummyC -Encoding utf8
    Write-Host "  -> Created dummy.c stubs" -ForegroundColor Green
}

# ---- Step 4: Check for C compiler ----
Write-Host ""
Write-Host "Checking for C compiler..." -ForegroundColor Yellow

$hasClang = $false
$hasGcc = $false

try { clang --version 2>$null | Out-Null; $hasClang = $true } catch {}
try { gcc --version 2>$null | Out-Null; $hasGcc = $true } catch {}

if ($hasClang) {
    Write-Host "  -> Found clang!" -ForegroundColor Green
} elseif ($hasGcc) {
    Write-Host "  -> Found gcc!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "WARNING: No C compiler found!" -ForegroundColor Red
    Write-Host "You need clang or gcc to compile DOOM." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Install clang with one of these commands:" -ForegroundColor White
    Write-Host "  winget install LLVM.LLVM" -ForegroundColor Cyan
    Write-Host "  scoop install llvm" -ForegroundColor Cyan
    Write-Host "  choco install llvm" -ForegroundColor Cyan
    Write-Host ""
}

# ---- Done! ----
Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  DOOM Setup Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor White
Write-Host "  1. Make sure clang/gcc is installed (see above)" -ForegroundColor Gray
Write-Host "  2. Build the kernel:  cargo bootimage" -ForegroundColor Cyan
Write-Host "  3. Run in QEMU:       Launch_QEMU_OS.bat" -ForegroundColor Cyan
Write-Host "  4. Open DOOM from the desktop and RIP AND TEAR!" -ForegroundColor Yellow
Write-Host ""

Read-Host "Press Enter to exit"
