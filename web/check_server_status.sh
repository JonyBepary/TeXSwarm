#!/bin/bash

# Check the status of the TeXSwarm backend servers

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Checking TeXSwarm server status...${NC}"

# Check if the HTTP API server is running
http_url="http://localhost:8080"
echo -n "HTTP API Server (${http_url}): "

if curl --output /dev/null --silent --head --fail "${http_url}"; then
    echo -e "${GREEN}RUNNING${NC}"
else
    echo -e "${RED}NOT RUNNING${NC}"
fi

# Check if the WebSocket server is running
# We can only check if the port is open, not if the WebSocket is working
ws_host="localhost"
ws_port="8081"
echo -n "WebSocket Server (${ws_host}:${ws_port}): "

if nc -z ${ws_host} ${ws_port} 2>/dev/null; then
    echo -e "${GREEN}RUNNING${NC}"
else
    echo -e "${RED}NOT RUNNING${NC}"
fi

echo ""
echo -e "${YELLOW}If servers are not running, start them with:${NC}"
echo -e "  cd /home/jony/rust-pract/TeXSwarm"
echo -e "  cargo run --bin server"
echo ""
echo -e "${YELLOW}To test connectivity from the web app:${NC}"
echo -e "  cd /home/jony/rust-pract/TeXSwarm/web"
echo -e "  ./serve_with_cors.sh"
echo -e "  # Then open http://localhost:8000/connection_test.html in your browser"
