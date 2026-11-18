@echo off
rem ============================================================
rem  Import Minecraft Java 26.3 resources from YOUR OWN copy.
rem  Horizon (and this package) contains no Minecraft assets.
rem  Requirements: Minecraft Java 26.3 played once from the official
rem  launcher, plus a Java 25 runtime (JDK recommended).
rem ============================================================
setlocal
set "PKGREL=%~dp0"
set "GAME=%PKREL%game"
set "PREP=%GAME%\horizon-prepare.exe"

echo.
echo  Horizon Ports - resource preparation
echo ------------------------------------------------------------
echo  This imports resources from your own Minecraft Java Edition.
echo  Nothing is downloaded or bundled from Mojang by this package.
echo.

if not exist "%PREP%" (
  echo  [ERROR] Missing horizon-prepare.exe at:
  echo    %PREP%
  echo.
  pause
  exit /b 1
)

rem ---- defaults, change them here if you like ----
set "MC_DIR=E:\Minecraft\.minecraft"
set "SERVER_JAR=E:\Minecraft\.minecraft\server-26.3\server.jar"
set "JAVA25=E:\Minecraft\jdk-25\jdk-25.0.5.0.2+1"

echo  Press Enter to accept the value in brackets.
echo.
set /p "MC_DIR=    Minecraft .minecraft folder [%MC_DIR%]: "
set /p "SERVER_JAR=Pinned 26.3 server.jar      [%SERVER_JAR%]: "
set /p "JAVA25=    Java 25 home                 [%JAVA25%]: "
echo.

if not exist "%MC_DIR%\assets\indexes" (
  echo  [ERROR] No assets\indexes under: %MC_DIR%
  echo  Install and play Minecraft Java 26.3 from the official launcher first.
  echo.
  pause
  exit /b 1
)
if not exist "%SERVER_JAR%" (
  echo  [ERROR] server.jar not found: %SERVER_JAR%
  echo  Get the pinned 26.3 server jar named in your launcher's 26.3.json.
  echo.
  pause
  exit /b 1
)
if not exist "%JAVA25%\bin\java.exe" (
  echo  [ERROR] java.exe not found under: %JAVA25%\bin
  echo  Install a Java 25 JDK / runtime and pass its home above.
  echo.
  pause
  exit /b 1
)

set "JAVA_HOME=%JAVA25%"
set "PATH=%JAVA25%\bin;%PATH%"

echo  Importing resources (this can take several minutes)...
echo.
"%PREP%" --prepare --server-jar "%SERVER_JAR%" --minecraft-dir "%MC_DIR%" --java-home "%JAVA25%" --root "%GAME%" --complete
if errorlevel 1 (
  echo.
  echo  [ERROR] Preparation failed. Read the messages above.
  echo  You can rerun this script; already-imported files are reused.
  echo.
  pause
  exit /b 1
)

echo.
echo  Done. Verifying resources...
"%PREP%" --check --root "%GAME%"
if errorlevel 1 (
  echo  [WARN] --check reported gaps. Re-run this script once more.
) else (
  echo  [OK] Resources complete. You can now run START_GAME.cmd
)
echo.
pause
endlocal
