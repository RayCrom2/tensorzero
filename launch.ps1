# TensorZero Quick Launch Script
# Run this script from the project root directory

Write-Host "Starting TensorZero Project..." -ForegroundColor Green

# Set environment variables
Write-Host "Setting environment variables..." -ForegroundColor Yellow
$env:NVIDIA_API_KEY="nvapi-your-nvidia-api-key-here"
$env:OPENAI_API_KEY="sk-your-openai-api-key-here"
$env:TENSORZERO_CLICKHOUSE_URL="https://clickhouse-cloud-url-here"

# Stop any existing services first
Write-Host "Stopping any existing services..." -ForegroundColor Yellow
docker compose down

# Start all services
Write-Host "Starting all Docker services..." -ForegroundColor Yellow
docker compose up -d

# Wait for services to start
Write-Host "Waiting for services to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Check status
Write-Host "Checking service status..." -ForegroundColor Yellow
docker compose ps

# Check gateway logs
Write-Host "Gateway logs:" -ForegroundColor Yellow
docker logs tensorzero-gateway-1 --tail 5

# Check UI logs
Write-Host "UI logs:" -ForegroundColor Yellow
docker logs tensorzero-ui-1 --tail 5

Write-Host "`nLaunch complete!" -ForegroundColor Green
Write-Host "Gateway: http://localhost:3000" -ForegroundColor Cyan
Write-Host "UI: http://localhost:4000" -ForegroundColor Cyan
Write-Host "ClickHouse: http://localhost:8123" -ForegroundColor Cyan

Write-Host "`nTo test the setup, run: python test.py" -ForegroundColor Magenta
