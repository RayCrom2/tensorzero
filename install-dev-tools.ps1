# TensorZero Development Tools Installation Script for Windows
# Run this script as Administrator for best results

Write-Host "Installing TensorZero development tools..." -ForegroundColor Green

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning "For best results, run this script as Administrator"
}

# Install UV (Python package manager)
Write-Host "Installing UV..." -ForegroundColor Yellow
try {
    irm https://astral.sh/uv/install.ps1 | iex
    Write-Host "UV installed successfully" -ForegroundColor Green
} catch {
    Write-Warning "Failed to install UV. You may need to install it manually."
    Write-Host "Visit: https://docs.astral.sh/uv/getting-started/installation/"
}

# Check for Node.js
Write-Host "Checking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "Node.js is installed: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Warning "Node.js not found. Please install from https://nodejs.org/"
    Write-Host "After installing Node.js, rerun this script."
    exit 1
}

# Install PNPM
Write-Host "Installing PNPM..." -ForegroundColor Yellow
try {
    npm install -g pnpm@10.12.4
    Write-Host "PNPM installed successfully" -ForegroundColor Green
} catch {
    Write-Warning "Failed to install PNPM"
}

# Check for Rust
Write-Host "Checking Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "Rust is installed: $rustVersion" -ForegroundColor Green
} catch {
    Write-Warning "Rust not found. Installing Rust..."
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
        .\rustup-init.exe -y --default-toolchain stable
        Remove-Item "rustup-init.exe"

        # Add Rust to PATH for current session
        $env:PATH += ";$env:USERPROFILE\.cargo\bin"

        Write-Host "Rust installed successfully" -ForegroundColor Green
        Write-Host "You may need to restart your terminal for Rust commands to work" -ForegroundColor Yellow
    } catch {
        Write-Warning "Failed to install Rust. Please install manually from https://rustup.rs/"
    }
}

# Install pre-commit
Write-Host "Installing pre-commit..." -ForegroundColor Yellow
try {
    uv tool install pre-commit
    Write-Host "Pre-commit installed successfully" -ForegroundColor Green
} catch {
    Write-Warning "Failed to install pre-commit with UV. Trying pip..."
    try {
        pip install pre-commit
        Write-Host "Pre-commit installed with pip" -ForegroundColor Green
    } catch {
        Write-Warning "Failed to install pre-commit"
    }
}

Write-Host "`nInstallation complete!" -ForegroundColor Green
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Restart your terminal/PowerShell session" -ForegroundColor White
Write-Host "2. Navigate to your TensorZero directory" -ForegroundColor White
Write-Host "3. Run: pre-commit install" -ForegroundColor White
Write-Host "4. Try your git commit again" -ForegroundColor White

Write-Host "`nTo verify installations, run:" -ForegroundColor Cyan
Write-Host "uv --version" -ForegroundColor White
Write-Host "pnpm --version" -ForegroundColor White
Write-Host "cargo --version" -ForegroundColor White
Write-Host "pre-commit --version" -ForegroundColor White
