@echo off
setlocal enabledelayedexpansion

set TAG=%1
if "%TAG%"=="" set TAG=v1.2.3

echo ========================================================
echo   LinkCam - Git Push and Release Automation (!TAG!)
echo ========================================================
echo Target Remote: https://github.com/Aniketkumar-01/LinkCam.git
echo.

REM 1. Initialize git if needed
if not exist .git (
    echo [*] Initializing Git repository...
    git init
) else (
    echo [*] Git repository already initialized.
)

REM 2. Set default branch to main
echo [*] Setting default branch to main...
git branch -M main

REM 3. Configure remote origin
echo [*] Configuring remote origin...
git remote remove origin 2>nul
git remote add origin https://github.com/Aniketkumar-01/LinkCam.git

REM 4. Stage all files
echo [*] Staging files...
git add .

REM 5. Commit changes
echo [*] Committing !TAG! release...
git commit -m "release: !TAG! - DirectShow COM filter build, signed Android release, and transport encryption"

REM 6. Push to main branch
echo [*] Pushing to GitHub (main)...
git push -u origin main
if %errorlevel% neq 0 (
    echo [!] Push failed. Please verify your GitHub credentials or permissions.
    pause
    exit /b %errorlevel%
)

REM 7. Create and push tag
echo [*] Tagging !TAG! release...
git tag -d !TAG! 2>nul
git tag -a !TAG! -m "Release !TAG!: Continuity Camera Companion for Android and Windows"
git push origin !TAG! --force

REM 8. Create GitHub release via gh CLI if available
where gh >nul 2>nul
if %errorlevel% equ 0 (
    echo [*] Creating GitHub Release via gh CLI...
    gh release create !TAG! --title "LinkCam !TAG! - Continuity Camera Companion" --notes-file CHANGELOG.md
    if %errorlevel% equ 0 (
        echo [OK] GitHub Release !TAG! successfully created!
    )
) else (
    echo [i] GitHub CLI (gh) not detected. Tag !TAG! has been pushed. You can publish the release notes directly at:
    echo     https://github.com/Aniketkumar-01/LinkCam/releases/new?tag=!TAG!
)

echo.
echo ========================================================
echo   Push & Release completed successfully!
echo   https://github.com/Aniketkumar-01/LinkCam
echo ========================================================
pause
