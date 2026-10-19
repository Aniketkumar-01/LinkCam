Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  LinkCam - Git Push & Release Automation" -ForegroundColor Cyan
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

Write-Host "[*] Committing v1.0.0 release..." -ForegroundColor Yellow
git commit -m "feat: initial release v1.0.0 (LinkCam Continuity Companion)"

# 4. Push main
Write-Host "[*] Pushing to GitHub (main)..." -ForegroundColor Yellow
git push -u origin main

if ($LASTEXITCODE -ne 0) {
    Write-Host "[!] Push failed. Please check your GitHub credentials." -ForegroundColor Red
    exit $LASTEXITCODE
}

# 5. Tag and Push Tag
Write-Host "[*] Tagging v1.0.0..." -ForegroundColor Yellow
git tag -d v1.0.0 2>$null
git tag -a v1.0.0 -m "Release v1.0.0: Continuity Camera Companion for Android & Windows"
git push origin v1.0.0 --force

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
