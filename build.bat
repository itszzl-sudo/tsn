@echo off
echo 编译运行时库...
gcc -c runtime.c -o runtime.o -O2
if %ERRORLEVEL% EQU 0 (
    echo ✅ runtime.o 编译成功
) else (
    echo ❌ runtime.o 编译失败
)

echo.
echo 编译 ts-native...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    echo ✅ ts-native 编译成功
) else (
    echo ❌ ts-native 编译失败
)
