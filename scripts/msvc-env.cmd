@echo off
rem Sets up MSVC x64 environment + Rust toolchain on PATH, then runs a command.
rem Usage: msvc-env.cmd <cargo subcommand args...>
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
shift /1
cmd /c "%*"
