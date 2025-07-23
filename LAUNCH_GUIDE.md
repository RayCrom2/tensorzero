# TensorZero Project Launch Guide

## Prerequisites
- Docker Desktop installed and running
- Python environment set up
- All API keys available (OpenAI and NVIDIA)

## Step-by-Step Launch Process

### 1. Navigate to Project Directory
```powershell
cd "C:\Users\raymo\VScode\tensorzero"
```

### 2. Set Environment Variables (Required for local Python client)
```powershell
$env:NVIDIA_API_KEY="your_nvidia_api_key_here"
$env:OPENAI_API_KEY="your_openai_api_key_here"
$env:TENSORZERO_CLICKHOUSE_URL="http://chuser:chpassword@localhost:8123"
```

### 3. Start All Services
```powershell
# Start all services (ClickHouse, Gateway, and UI)
docker compose up -d

# Or to see logs in real-time
docker compose up
```

### 4. Verify Services are Running
```powershell
docker ps
```

You should see all three services:
- `tensorzero-clickhouse-1` on ports 8123 and 9000
- `tensorzero-gateway-1` on port 3000
- `tensorzero-ui-1` on port 4000

### 5. Check Gateway Health
```powershell
# Check logs to ensure no errors
docker logs tensorzero-gateway-1

# Optional: Test health endpoint
curl http://localhost:3000/health
```

### 6. Test the Setup
```powershell
python test.py
```

## Service URLs
- **TensorZero Gateway**: http://localhost:3000
- **TensorZero UI**: http://localhost:4000
- **ClickHouse**: http://localhost:8123

## Quick Commands

### Restart Gateway Only
```powershell
docker compose restart gateway
```

### Stop All Services
```powershell
docker compose down
```

### View Gateway Logs
```powershell
docker logs tensorzero-gateway-1 --follow
```

### Check Service Status
```powershell
docker compose ps
```

## Troubleshooting

### If Gateway Fails to Start
1. Check logs: `docker logs tensorzero-gateway-1`
2. Verify config file: `config/tensorzero.toml`
3. Ensure environment variables are set
4. Restart: `docker compose restart gateway`

### If Python Test Fails
1. Ensure environment variables are set in current PowerShell session
2. Check that gateway is running: `docker ps`
3. Verify configuration file syntax

### If API Key Errors
1. Double-check API keys in environment variables
2. Ensure `.env` file exists with correct values
3. Restart gateway after fixing keys

## Configuration Files
- **Docker Compose**: `docker-compose.yml`
- **TensorZero Config**: `config/tensorzero.toml`
- **Environment Variables**: `.env`

## Model Variants Available
- `gpt-4o` (OpenAI GPT-4o)
- `gpt_4o_mini` (OpenAI GPT-4o Mini)
- `llama_3_1_8b` (NVIDIA NIM Llama 3.1 8B)

---

**Note**: Make sure Docker Desktop is running before executing any docker commands. The environment variables need to be set in each new PowerShell session unless you add them to your system environment variables permanently.
