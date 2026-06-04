@echo off
REM tsn 发布脚本
REM 从 ts-native 同步代码并发布到 crates.io

setlocal enabledelayedexpansion

REM === 读取版本号 ===
for /f "tokens=3 delims==\"" %%v in ('findstr "version" ..\ts-native\Cargo.toml ^| findstr /n "^" ^| findstr "^1:"') do set VERSION=%%v
if "%VERSION%"=="" (
    echo ERROR: Cannot read version from ts-native/Cargo.toml
    exit /b 1
)
echo Version: %VERSION%

REM === 1. 同步 ts-native → tsn ===
echo.
echo [1/3] Syncing ts-native to tsn...

for /d %%d in (*.*) do if not "%%d"==".git" rd /s /q "%%d"
del /q * 2>nul

xcopy ..\ts-native\* . /s /e /y

powershell -Command "(Get-Content Cargo.toml) -replace 'name = \"ts-native\"', 'name = \"tsn\"' | Set-Content Cargo.toml"

echo   Done.

REM === 2. 提交 ===
echo.
echo [2/3] Committing...
git add -A
git commit -m "release: v%VERSION%" --allow-empty
git push origin master
echo   Done.

REM === 3. 发布到 crates.io ===
echo.
echo [3/3] Publishing tsn to crates.io...
cargo publish --registry crates-io
echo   Done.

echo.
echo ========================================
echo   tsn v%VERSION% published!
echo ========================================

endlocal