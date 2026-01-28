@echo off
REM ARM Hypervisor Quick Start Script for Windows
REM Sets up testing environment and runs the project in Docker

setlocal enabledelayedexpansion

REM Colors using color codes (basic)
set "GREEN=[92m"
set "YELLOW=[93m"
set "RED=[91m"
set "NC=[0m"

REM Check prerequisites
echo.
echo %GREEN%====================================%NC%
echo %GREEN%ARM Hypervisor Quick Start Script%NC%
echo %GREEN%====================================%NC%
echo.

echo %YELLOW%[*]%NC% Checking prerequisites...

docker --version >nul 2>&1
if errorlevel 1 (
    echo %RED%[x]%NC% Docker is not installed or not in PATH
    echo Install Docker Desktop from https://www.docker.com/products/docker-desktop
    exit /b 1
)
for /f "tokens=*" %%i in ('docker --version') do echo %GREEN%[check]%NC% %%i

docker-compose --version >nul 2>&1
if errorlevel 1 (
    echo %RED%[x]%NC% Docker Compose is not installed
    echo It should be included with Docker Desktop
    exit /b 1
)
for /f "tokens=*" %%i in ('docker-compose --version') do echo %GREEN%[check]%NC% %%i

REM Parse arguments
if "%1"=="stop" (
    echo %YELLOW%[*]%NC% Stopping services...
    docker-compose down
    echo %GREEN%[check]%NC% Services stopped
    exit /b 0
)

if "%1"=="logs" (
    docker-compose logs -f api-server
    exit /b 0
)

if "%1"=="test" (
    echo %YELLOW%[*]%NC% Running tests inside container...
    docker-compose exec api-server cargo test --workspace
    exit /b 0
)

if "%1"=="shell" (
    docker-compose exec api-server /bin/bash
    exit /b 0
)

if "%1"=="build-only" (
    echo %YELLOW%[*]%NC% Building Docker image...
    docker build -t arm-hypervisor:latest .
    echo %GREEN%[check]%NC% Docker image built successfully
    exit /b 0
)

REM Default: build and start
echo %YELLOW%[*]%NC% Building Docker image...
docker build -t arm-hypervisor:latest .
if errorlevel 1 (
    echo %RED%[x]%NC% Docker build failed
    exit /b 1
)
echo %GREEN%[check]%NC% Docker image built successfully
echo.

echo %YELLOW%[*]%NC% Starting Docker Compose services...
docker-compose up -d
echo %GREEN%[check]%NC% Services started
echo.

echo %YELLOW%[*]%NC% Waiting for API server to be ready...
timeout /t 5 /nobreak

echo.
echo %GREEN%====================================%NC%
echo %GREEN%ARM Hypervisor is ready for testing!%NC%
echo %GREEN%====================================%NC%
echo.

echo Endpoint Information:
echo   API Server:  http://localhost:8080
echo   Logs:        docker-compose logs -f api-server
echo.

echo Test URLs:
echo   Health:      curl http://localhost:8080/health
echo   Containers:  curl http://localhost:8080/api/containers
echo   Cluster:     curl http://localhost:8080/api/cluster/status
echo.

echo Docker Commands:
echo   Stop:        docker-compose down
echo   Logs:        docker-compose logs -f api-server
echo   Shell:       docker-compose exec api-server bash
echo   Run tests:   docker-compose exec api-server cargo test
echo.

echo Documentation:
echo   Testing:     See TESTING.md
echo   Deployment:  See DEPLOYMENT.md
echo.
