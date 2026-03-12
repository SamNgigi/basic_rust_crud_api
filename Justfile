set dotenv-load:=true
set dotenv-filename:=".env"


# ----------------------------------------------------
# DOCKER COMPOSE SHORTCUTS
# ----------------------------------------------------
_dc:="docker compose -f docker-compose.dev.yaml"

# Start all services
dev_up:
  {{_dc}} up -d

# Start all services with build
dev_up_build:
  {{_dc}} up -d --build

# Stop all services
dev_down:
  {{_dc}} down --remove-orphans

# Stop and remove volumes (⚠️ destroys data)
dev_nuke:
  {{_dc}} down -v

# View logs (all services)
dev_logs *ARGS:
  {{_dc}} logs -f {{ARGS}}

# View app logs only
dev_logs_app:
  {{_dc}} logs -f app


