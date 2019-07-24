@echo off

cargo test
if errorlevel 1 goto error

cargo fmt --all -- --check
if errorlevel 1 goto error

cargo clippy --all
if errorlevel 1 goto error

exit /b 0

:error
echo Validation error
exit /b 1
