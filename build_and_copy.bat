@echo off
setlocal enabledelayedexpansion

REM Move to script directory (assumes script is in root)
REM cd /d %~dp0

echo Cleaning up previous binaries workspace...
cargo clean

echo Building all Rust projects in workspace...
cargo build --release

REM Create target and plugin folders
if not exist target (
    mkdir target
)

REM Copy workflow_engine.exe to target\
set ENGINE_EXE=target\release\oobe_engine.exe
if exist "%ENGINE_EXE%" (
    echo Copying oobe_engine.exe to target...
    copy /Y "%ENGINE_EXE%" target\
)

REM Copy plugin DLLs to target\
echo Copying plugin DLLs to target\...
for %%F in (target\release\plugin*.dll) do (
    echo Copying %%~nxF...
    copy /Y "%%F" target\
)

REM Remove unnecessary folders from target\
if exist target\.fingerprint (
    echo Removing .fingerprint folder from target...
    rmdir /S /Q target\.fingerprint
)

if exist target\release (
    echo Removing release folder from target...
    rmdir /S /Q target\release
)

echo Done.
