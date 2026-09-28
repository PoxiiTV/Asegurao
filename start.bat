@echo off
REM Arranca Asegurao en modo desarrollo (ventana + recarga en caliente).
cd /d "%~dp0"
echo Iniciando Asegurao en modo desarrollo...
call npm run tauri dev
pause
