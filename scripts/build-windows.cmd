@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 -no_logo
if errorlevel 1 exit /b %errorlevel%

if not defined CARGO_HOME goto cargo_on_path
"%CARGO_HOME%\bin\cargo.exe" %*
exit /b %errorlevel%

:cargo_on_path
cargo %*
