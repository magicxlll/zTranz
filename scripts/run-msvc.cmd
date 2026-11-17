@echo off
rem Initializes MSVC x64 env, puts Rust on PATH, then runs `cargo <args>` in this session.
rem Usage (from git-bash): cmd //c "scripts\run-msvc.cmd check --target x86_64-pc-windows-msvc"
set "VARS=C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d "%~dp0.." || exit /b 1
call "%VARS%" || exit /b 1
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
cargo %*
