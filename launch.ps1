# TensorZero Quick Launch Script
# Run this script from the project root directory

Write-Host "Starting TensorZero Project..." -ForegroundColor Green

# Check if Docker is running
Write-Host "Checking Docker status..." -ForegroundColor Yellow
try {
    docker version | Out-Null
    Write-Host " Docker is running" -ForegroundColor Green
} catch {
    Write-Host " Docker is not running!" -ForegroundColor Red
    Write-Host "Please start Docker Desktop and try again." -ForegroundColor Red
    exit 1
}

# Set environment variables (replace with your actual API keys)
Write-Host "Setting environment variables..." -ForegroundColor Yellow
$env:NVIDIA_API_KEY="your_nvidia_api_key_here"
$env:OPENAI_API_KEY="your_openai_api_key_here"
$env:TENSORZERO_CLICKHOUSE_URL="http://chuser:chpassword@localhost:8123"

# Stop any existing services first
Write-Host "Stopping any existing services..." -ForegroundColor Yellow
docker compose down 2>$null

# Start all services
Write-Host "Starting all Docker services..." -ForegroundColor Yellow
docker compose up -d

if ($LASTEXITCODE -eq 0) {
    Write-Host " Docker services started successfully" -ForegroundColor Green

    # Wait for services to start
    Write-Host "Waiting for services to start..." -ForegroundColor Yellow
    Start-Sleep -Seconds 10

    # Check status
    Write-Host "Checking service status..." -ForegroundColor Yellow
    docker compose ps

    Write-Host "`nLaunch complete!" -ForegroundColor Green
    Write-Host "Gateway: http://localhost:3000" -ForegroundColor Cyan
    Write-Host "UI: http://localhost:4000" -ForegroundColor Cyan
    Write-Host "ClickHouse: http://localhost:8123" -ForegroundColor Cyan

    Write-Host "`nTo test the setup, run: python test.py" -ForegroundColor Magenta
} else {
    Write-Host " Failed to start Docker services!" -ForegroundColor Red
    exit 1
}
