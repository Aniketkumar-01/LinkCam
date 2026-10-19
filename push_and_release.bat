@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo   LinkCam - Git Push and Release Automation
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
echo [*] Committing v1.0.0 release...
git commit -m "feat: initial release v1.0.0 (LinkCam Continuity Companion)"

REM 6. Push to main branch
echo [*] Pushing to GitHub (main)...
git push -u origin main
if %errorlevel% neq 0 (
    echo [!] Push failed. Please verify your GitHub credentials or permissions.
    pause
    exit /b %errorlevel%
)

REM 7. Create and push tag v1.0.0
echo [*] Tagging v1.0.0 release...
git tag -d v1.0.0 2>nul
git tag -a v1.0.0 -m "Release v1.0.0: Continuity Camera Companion for Android & Windows"
git push origin v1.0.0 --force

REM 8. Create GitHub release via gh CLI if available
where gh >nul 2>nul
if %errorlevel% equ 0 (
    echo [*] Creating GitHub Release via gh CLI...
    gh release create v1.0.0 --title "LinkCam v1.0.0 - Continuity Camera Companion" --notes-file CHANGELOG.md
    if %errorlevel% equ 0 (
        echo [OK] GitHub Release v1.0.0 successfully created!
    )
) else (
    echo [i] GitHub CLI (gh) not detected. Tag v1.0.0 has been pushed. You can publish the release notes directly at:
    echo     https://github.com/Aniketkumar-01/LinkCam/releases/new?tag=v1.0.0
)

echo.
echo ========================================================
echo   Push & Release completed successfully!
echo   https://github.com/Aniketkumar-01/LinkCam
echo ========================================================
pause
