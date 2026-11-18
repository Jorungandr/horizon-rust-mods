@echo off
rem Play the injected build: three chat mods compiled into horizon-client.
rem Starts the game with its own prepared 26.3 resources and a dedicated save
rem folder, so the launcher's install is untouched.
setlocal
set "ROOT=%~dp0"
set "CTL=%ROOT%build-win\horizon-src\target\fast\horizon-ctl.exe"
set "GAME=%ROOT%build-win\game-port\horizon-client.exe"

if not exist "%GAME%" (
  echo Missing %GAME%
  echo Build it first: bash tools\build_win.sh, then: python tools\make_game_port.py
  pause
  exit /b 1
)
if not exist "%CTL%" set "CTL=%APPDATA%\horizon-launcher\versions\nightly-2026-11-13-9f42064\horizon-ctl.exe"

"%CTL%" launch --game "%GAME%" --world "Ports Proof" --resources "%ROOT%build-win\game-port\.local\horizon-client\26.3" --game-dir "%ROOT%build-win\game-port\gamedata" --home "%ROOT%build-win\game-port\gamelogs" --render-distance 8
if errorlevel 1 pause
endlocal
