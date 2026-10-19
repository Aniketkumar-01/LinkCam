param(
    [string]$Tag = "v1.0.5",
    [string]$Message = "Release v1.0.5: Remove unnecessary ndk block, clean gradle network config, and add CI failure dump"
)

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  LinkCam - Git Push & Release Automation ($Tag)" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "Target Remote: https://github.com/Aniketkumar-01/LinkCam.git`n"

# 1. Initialize git
if (-not (Test-Path ".git")) {
    Write-Host "[*] Initializing Git repository..." -ForegroundColor Yellow
    git init
} else {
    Write-Host "[*] Git repository already initialized." -ForegroundColor Green
}

# 2. Branch & Remote
Write-Host "[*] Setting branch to main and configuring origin..." -ForegroundColor Yellow
git branch -M main
git remote remove origin 2>$null
git remote add origin https://github.com/Aniketkumar-01/LinkCam.git

# 3. Add & Commit
Write-Host "[*] Staging files..." -ForegroundColor Yellow
git add .

Write-Host "[*] Committing $Tag release..." -ForegroundColor Yellow
git commit -m "fix(android): fix kotlin syntax error, add launcher icons, and stabilize CI build ($Tag)"

# 4. Push main
Write-Host "[*] Pushing to GitHub (main)..." -ForegroundColor Yellow
git push -u origin main

if ($LASTEXITCODE -ne 0) {
    Write-Host "[!] Push failed. Please check your GitHub credentials." -ForegroundColor Red
    exit $LASTEXITCODE
}

# 5. Tag and Push Tag
Write-Host "[*] Tagging $Tag..." -ForegroundColor Yellow
git tag -d $Tag 2>$null
git tag -a $Tag -m "$Message"
git push origin $Tag --force

# 6. GitHub Release via gh CLI if installed
if (Get-Command gh -ErrorAction SilentlyContinue) {
    Write-Host "[*] Creating GitHub Release via gh CLI..." -ForegroundColor Yellow
    gh release create v1.0.0 --title "LinkCam v1.0.0 - Continuity Camera Companion" --notes-file CHANGELOG.md
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] GitHub Release v1.0.0 successfully created!" -ForegroundColor Green
    }
} else {
    Write-Host "[i] GitHub CLI (gh) not detected. Tag v1.0.0 pushed. Publish release at:" -ForegroundColor Cyan
    Write-Host "    https://github.com/Aniketkumar-01/LinkCam/releases/new?tag=v1.0.0" -ForegroundColor Cyan
}

Write-Host "`n[DONE] Project pushed and released on https://github.com/Aniketkumar-01/LinkCam" -ForegroundColor Green
