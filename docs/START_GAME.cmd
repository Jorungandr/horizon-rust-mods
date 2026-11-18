@echo off
rem ============================================================
rem  Horizon Ports - one-click launcher for the injected build
rem  Six Rust mods are compiled into horizon-client.exe already.
rem  Run PREPARE_RESOURCES.cmd once before the first launch.
rem ============================================================
setlocal
set "PKGREL=%~dp0"
set "GAME=%PKREL%game"
set "CTL=%GAME%\horizon-ctl.exe"
set "CLIENT=%GAME%\horizon-client.exe"
set "RES=%GAME%\.local\horizon-client\26.3"

echo.
echo  Horizon Ports - launching with 6 Rust mods
echo ------------------------------------------------------------

if not exist "%CLIENT%" (
  echo  [ERROR] Missing game client:
  echo    %CLIENT%
  echo  The package is incomplete. Re-extract the archive.
  echo.
  pause
  exit /b 1
)
if not exist "%CTL%" (
  echo  [ERROR] Missing horizon-ctl.exe
  echo    %CTL%
  echo.
  pause
  exit /b 1
)
if not exist "%RES%\horizon-resources.json" (
  echo  [ERROR] Game resources are not prepared yet.
  echo    Expected: %RES%\horizon-resources.json
  echo.
  echo  Run PREPARE_RESOURCES.cmd first. It imports the 26.3 resources
  echo  from your own Minecraft Java Edition (Horizon ships none).
  echo.
  pause
  exit /b 1
)

echo  Client : %CLIENT%
echo  Resources: %RES%
echo  World  : Ports Proof   (change START_GAME.cmd to use another name)
echo.
echo  Controls: WASD move, mouse look, Space jump, F1 hide HUD,
echo            E inventory, Esc pause.  Options ^> Mods lists all six mods.
echo.

"%CTL%" launch --game "%CLIENT%" --world "Ports Proof" --resources "%RES%" --game-dir "%GAME%\gamedata" --home "%GAME%\gamelogs" --render-distance 8
if errorlevel 1 (
  echo.
  echo  The game exited with an error. See %GAME%\gamelogs\logs for details.
  pause
)
endlocal
