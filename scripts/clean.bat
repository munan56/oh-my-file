@echo off
REM Clean script for oh-my-file (Windows)
REM Removes build artifacts and caches

setlocal

set "PROJECT_ROOT=%~dp0.."

echo.
echo 🧹 Cleaning build artifacts...
echo.

cd /d "%PROJECT_ROOT%"

if exist dist (
  echo ✓ Removing dist...
  rmdir /s /q dist
)

if exist src-tauri\target (
  echo ✓ Removing src-tauri\target...
  rmdir /s /q src-tauri\target
)

echo.
echo ✅ Cleanup complete!
