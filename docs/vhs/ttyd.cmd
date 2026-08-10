@echo off
setlocal

rem Windows-only workaround for a long-standing VHS/ttyd bug where VHS never
rem passes ttyd a -w (writable working dir) flag, which makes ttyd hang
rem silently on Windows: https://github.com/charmbracelet/vhs/issues/631
rem This shim finds the real ttyd.exe and re-invokes it with -w set to an
rem absolute path (ttyd rejects expansions like ~/Desktop).

set "ttydExe="
for /f "delims=" %%I in ('where ttyd.exe 2^>nul') do if not defined ttydExe set "ttydExe=%%I"

if not defined ttydExe (
	echo ttyd.exe was not found on PATH.
	exit /b 1
)

"%ttydExe%" -w "%CD%" %*
