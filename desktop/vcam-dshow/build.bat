@echo off
REM Direct compilation with MSVC Developer Command Prompt
cl.exe /O2 /LD /std:c++17 LinkCamDShow.cpp /link /DEF:LinkCamDShow.def /OUT:LinkCamDShow64.dll strmiids.lib ole32.lib oleaut32.lib user32.lib advapi32.lib mfplat.lib mfuuid.lib
if %ERRORLEVEL% equ 0 (
    echo [LinkCam] Successfully compiled LinkCamDShow64.dll
)
