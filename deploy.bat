@echo off
setlocal
REM Compila Asegurao para produccion y copia el instalador a deploy-hosting.
cd /d "%~dp0"

REM Clave de firma de actualizaciones (TAURI_SIGNING_PRIVATE_KEY...) desde .env.
if exist ".env" for /f "usebackq eol=# tokens=1,* delims==" %%a in (".env") do set "%%a=%%b"

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

for /f "tokens=2 delims=:, " %%v in ('findstr /c:"\"version\"" "%~dp0src-tauri\tauri.conf.json"') do set "VER=%%~v"

REM Instalador NSIS de esta version + su firma de actualizacion (.sig).
copy /y "%~dp0src-tauri\target\release\bundle\nsis\Asegurao_%VER%_x64-setup.exe*" "%OUT%" >nul

REM Portable: el .exe de release ya lleva el frontend e icono embebidos.
copy /y "%~dp0src-tauri\target\release\asegurao.exe" "%OUT%\Asegurao_%VER%_x64-portable.exe" >nul

REM Manifiesto del actualizador (se sube a la release junto al setup).
call node scripts\latest-json.mjs

echo.
echo Listo. Archivos en: %OUT%
dir /b "%OUT%"
pause
