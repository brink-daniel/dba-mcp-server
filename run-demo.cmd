@echo off
setlocal
cd /d "%~dp0"

rem Demo-only token. Do not use this token on an untrusted network.
cargo run -- --token dba-mcp-server-demo-token
exit /b %ERRORLEVEL%
