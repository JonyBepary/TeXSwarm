#!/bin/bash

# This script serves the TeXSwarm web interface and handles CORS issues

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting TeXSwarm Web Server${NC}"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo -e "${RED}Error: npm is not installed. Please install Node.js and npm.${NC}"
    exit 1
fi

# Create package.json if it doesn't exist
if [ ! -f "package.json" ]; then
    echo -e "${YELLOW}Creating package.json...${NC}"
    echo '{
  "name": "texswarm-web",
  "version": "1.0.0",
  "description": "TeXSwarm Web Interface",
  "private": true
}' > package.json
fi

# Install http-server locally if node_modules/http-server doesn't exist
if [ ! -d "node_modules/http-server" ]; then
    echo -e "${YELLOW}Installing http-server locally...${NC}"
    npm install --save-dev http-server
fi

# Read API port from config.json if available
CONFIG_FILE="../config.json"
API_PORT=8090  # Default API port

if [ -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Reading configuration from ${CONFIG_FILE}${NC}"
    if command -v jq &> /dev/null; then
        # Use jq if available for reliable JSON parsing
        API_PORT=$(jq -r '.server.api_port' "$CONFIG_FILE" 2>/dev/null || echo 8090)
    else
        # Fallback to grep for basic extraction
        API_PORT=$(grep -o '"api_port": *[0-9]*' "$CONFIG_FILE" 2>/dev/null | awk '{print $2}' || echo 8090)
    fi
    echo -e "${YELLOW}Found API port: ${API_PORT}${NC}"
fi

# Check if the backend server is running
backend_url="http://localhost:${API_PORT}"
echo -e "${YELLOW}Checking if backend server is running at ${backend_url}...${NC}"

if curl --output /dev/null --silent --head --fail "${backend_url}/api/ping"; then
    echo -e "${GREEN}Backend server is running!${NC}"
else
    echo -e "${RED}Warning: Backend server does not appear to be running at ${backend_url}.${NC}"
    echo -e "${YELLOW}Make sure the backend server is running with 'cargo run --bin server'${NC}"
    echo -e "${YELLOW}Continuing anyway, but the web interface may not work correctly...${NC}"
fi

# Start http-server with CORS enabled
echo -e "${GREEN}Starting web server with CORS enabled...${NC}"

# Set port to a higher range to avoid conflicts
PORT=3000

echo -e "${GREEN}Starting server on port $PORT...${NC}"
echo -e "${GREEN}Access the web interface at: http://localhost:$PORT${NC}"
echo -e "${GREEN}Access the modern editor at: http://localhost:$PORT/new-index.html${NC}"
echo -e "${GREEN}Access the connection test at: http://localhost:$PORT/connection_test.html${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"

# Start the server
npx http-server . --cors -p $PORT -c-1
