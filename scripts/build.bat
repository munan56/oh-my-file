@echo off
REM Build script for oh-my-file (Windows)
REM Usage: scripts\build.bat [TARGET]

setlocal enabledelayedexpansion

set "PROJECT_ROOT=%~dp0.."
set "TARGET=%1"

if "%TARGET%"=="" set "TARGET=all"

echo.
echo 🔨 Building oh-my-file
echo 📁 Project root: %PROJECT_ROOT%
echo 🎯 Target: %TARGET%
echo.

cd /d "%PROJECT_ROOT%"

if "%TARGET%"=="all" (
  echo 📝 Building for Windows...
  call npm run package win32
  goto :end
)

if "%TARGET%"=="win32" (
  echo 📝 Building for Windows...
  call npm run package win32
  goto :end
)

if "%TARGET%"=="darwin" (
  echo ❌ Cannot build for macOS on Windows
  echo    Run on a macOS machine instead
  exit /b 1
)

if "%TARGET%"=="linux" (
  echo ❌ Cannot build for Linux on Windows
  echo    Run on a Linux machine instead
  exit /b 1
)

echo ❌ Unknown target: %TARGET%
echo Available targets: all, win32, darwin, linux
exit /b 1

:end
echo.
echo ✅ Build complete!
