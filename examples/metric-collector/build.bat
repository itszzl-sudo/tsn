@echo off
REM 编译脚本 - Windows

echo 编译 TS-Native 运维采集CLI...

REM 编译为原生二进制
ts-native main.ts

REM 重命名输出
if exist a.exe (
    move a.exe metric-collector.exe
    echo ✅ 编译成功: metric-collector.exe
    dir metric-collector.exe
) else (
    echo ❌ 编译失败
    exit /b 1
)