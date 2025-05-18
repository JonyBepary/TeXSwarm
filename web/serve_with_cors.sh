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

# Install http-server if not already installed
if ! command -v http-server &> /dev/null; then
    echo -e "${YELLOW}Installing http-server...${NC}"
    npm install -g http-server
fi

# Check if the backend server is running
backend_url="http://localhost:8080"
echo -e "${YELLOW}Checking if backend server is running at ${backend_url}...${NC}"

if curl --output /dev/null --silent --head --fail "${backend_url}"; then
    echo -e "${GREEN}Backend server is running!${NC}"
else
    echo -e "${RED}Warning: Backend server does not appear to be running at ${backend_url}.${NC}"
    echo -e "${YELLOW}Make sure the backend server is running with 'cargo run --bin server'${NC}"
    echo -e "${YELLOW}Continuing anyway, but the web interface may not work correctly...${NC}"
fi

# Start http-server with CORS enabled
echo -e "${GREEN}Starting web server with CORS enabled...${NC}"
echo -e "${GREEN}Access the web interface at: http://localhost:8000${NC}"
echo -e "${GREEN}Access the connection test at: http://localhost:8000/connection_test.html${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"

http-server . --cors -p 8000
