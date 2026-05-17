@echo off
REM Development server script for oh-my-file (Windows)
REM Starts the Tauri dev server with hot reload

setlocal

set "PROJECT_ROOT=%~dp0.."

echo.
echo 🚀 Starting development server...
echo 📁 Project root: %PROJECT_ROOT%
echo.

cd /d "%PROJECT_ROOT%"

call npm run tauri dev

echo.
echo ✅ Dev server stopped
