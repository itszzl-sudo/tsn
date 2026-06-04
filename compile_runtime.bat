@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\Administrator\Documents\codebuddy-projects\ts-native
echo Compiling runtime_nocrt.c...
cl /c /O2 runtime_nocrt.c /Fornuntime_nocrt.o
if %ERRORLEVEL% NEQ 0 exit /b 1
echo Compiling start_nocrt.c...
cl /c /O2 start_nocrt.c /Fostart_nocrt.o
if %ERRORLEVEL% NEQ 0 exit /b 1
echo Done.
