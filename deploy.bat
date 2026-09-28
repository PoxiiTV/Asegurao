@echo off
setlocal
REM Compila Asegurao para produccion y copia el instalador a deploy-hosting.
cd /d "%~dp0"

echo ============================================
echo   Compilando Asegurao (release + instalador)
echo ============================================
call npm run tauri build
if errorlevel 1 (
  echo.
  echo ERROR: la compilacion ha fallado.
  pause
  exit /b 1
)

set "OUT=%~dp0deploy-hosting"
if exist "%OUT%" rmdir /s /q "%OUT%"
mkdir "%OUT%"

REM Instalador NSIS + datos de auto-actualizacion (latest.json / .sig si existen).
set "NSIS=%~dp0src-tauri\target\release\bundle\nsis"
if exist "%NSIS%" (
  copy /y "%NSIS%\*.exe" "%OUT%" >nul 2>&1
  copy /y "%NSIS%\*.sig" "%OUT%" >nul 2>&1
)

REM Portable: el .exe de release ya lleva el frontend e icono embebidos.
for /f "tokens=2 delims=:, " %%v in ('findstr /c:"\"version\"" "%~dp0src-tauri\tauri.conf.json"') do set "VER=%%~v"
copy /y "%~dp0src-tauri\target\release\asegurao.exe" "%OUT%\Asegurao_%VER%_x64-portable.exe" >nul

echo.
echo Listo. Archivos en: %OUT%
dir /b "%OUT%"
pause
