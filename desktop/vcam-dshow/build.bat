@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo [LinkCam] Preparing DirectShow Virtual Camera Filter build...

where cl.exe >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [LinkCam] Locating MSVC x64 build tools...
    set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
    set "VCVARS="

    if exist "!VSWHERE!" (
        for /f "usebackq tokens=*" %%i in (`"!VSWHERE!" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
            if exist "%%i\VC\Auxiliary\Build\vcvars64.bat" (
                set "VCVARS=%%i\VC\Auxiliary\Build\vcvars64.bat"
            )
        )
    )

    if not defined VCVARS (
        for %%v in (
            "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
            "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
            "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
            "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
            "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
            "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
        ) do (
            if exist %%v (
                set "VCVARS=%%~v"
                goto :found_vcvars
            )
        )
    )

:found_vcvars
    if defined VCVARS (
        echo [LinkCam] Initializing MSVC environment via: "!VCVARS!"
        call "!VCVARS!"
    ) else (
        echo [LinkCam] ERROR: Microsoft Visual C++ Build Tools (cl.exe) not found.
        echo [LinkCam] Please install Visual Studio with "Desktop development with C++" or run from Developer Command Prompt.
        exit /b 1
    )
)

echo [LinkCam] Compiling LinkCamDShow64.dll...
cl.exe /O2 /LD /std:c++17 /EHsc LinkCamDShow.cpp /link /DEF:LinkCamDShow.def /OUT:LinkCamDShow64.dll strmiids.lib ole32.lib oleaut32.lib user32.lib advapi32.lib mfplat.lib mfuuid.lib wmcodecdspuuid.lib

if %ERRORLEVEL% equ 0 (
    echo [LinkCam] Successfully compiled LinkCamDShow64.dll
    regsvr32.exe /s "%~dp0LinkCamDShow64.dll"
    echo [LinkCam] Registered LinkCamDShow64.dll with DirectShow and COM.

    if exist "..\src-tauri\target\release" (
        copy /y LinkCamDShow64.dll "..\src-tauri\target\release\LinkCamDShow64.dll" >nul
        echo [LinkCam] Copied to desktop release directory.
    )

    if exist "D:\Programs\LinkCam" (
        copy /y LinkCamDShow64.dll "D:\Programs\LinkCam\LinkCamDShow64.dll" >nul
        regsvr32.exe /s "D:\Programs\LinkCam\LinkCamDShow64.dll"
        echo [LinkCam] Copied and registered with D:\Programs\LinkCam
    )
) else (
    echo [LinkCam] Compilation failed with error code %ERRORLEVEL%.
    exit /b %ERRORLEVEL%
)
