@echo off
setlocal enabledelayedexpansion

set TSN=E:\Administrator\Documents\codebuddy-projects\ts-native\target\release\ts-native.exe
set EXAMPLES=E:\Administrator\Documents\codebuddy-projects\ts-native\examples
set OUTDIR=E:\Administrator\Documents\codebuddy-projects\ts-native\examples\output

if not exist "%OUTDIR%" mkdir "%OUTDIR%"

echo ========================================
echo Compiling examples with ts-native.exe
echo ========================================
echo.

REM 1. simple.ts
echo [1] Compiling simple.ts...
%TSN% "%EXAMPLES%\test-size\simple.ts" -o "%OUTDIR%\simple.o" 2>&1
echo.

REM 2. main-v2.ts
echo [2] Compiling main-v2.ts...
%TSN% "%EXAMPLES%\metric-collector-example\metric-collector\main-v2.ts" -o "%OUTDIR%\main-v2.o" 2>&1
echo.

REM 3. main-simple.ts
echo [3] Compiling main-simple.ts...
%TSN% "%EXAMPLES%\metric-collector-example\metric-collector\main-simple.ts" -o "%OUTDIR%\main-simple.o" 2>&1
echo.

REM 4. main.ts (complex)
echo [4] Compiling main.ts (complex)...
%TSN% "%EXAMPLES%\metric-collector-example\metric-collector\main.ts" -o "%OUTDIR%\main.o" 2>&1
echo.

echo ========================================
echo Done. Check examples\output\ for results.
echo ========================================
dir "%OUTDIR%"
