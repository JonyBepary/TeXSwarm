#!/bin/bash

# Check the status of the TeXSwarm backend servers

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT=$(cd "$(dirname "$0")/.." && pwd)
CONFIG_FILE="${PROJECT_ROOT}/config.json"

echo -e "${YELLOW}Checking TeXSwarm server status...${NC}"

# Read port values from config.json
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Reading configuration from ${CONFIG_FILE}${NC}"
    API_HOST=$(grep -o '"api_host": *"[^"]*"' "$CONFIG_FILE" | cut -d'"' -f4)
    API_PORT=$(grep -o '"api_port": *[0-9]*' "$CONFIG_FILE" | awk '{print $2}')
    WS_HOST=$(grep -o '"ws_host": *"[^"]*"' "$CONFIG_FILE" | cut -d'"' -f4)
    WS_PORT=$(grep -o '"ws_port": *[0-9]*' "$CONFIG_FILE" | awk '{print $2}')
    DOC_HOST=$(grep -o '"document_host": *"[^"]*"' "$CONFIG_FILE" | cut -d'"' -f4)
    DOC_PORT=$(grep -o '"document_port": *[0-9]*' "$CONFIG_FILE" | awk '{print $2}')

    # For client-side connections, use localhost instead of 0.0.0.0
    if [ "$API_HOST" == "0.0.0.0" ]; then
        CLIENT_API_HOST="localhost"
    else
        CLIENT_API_HOST="$API_HOST"
    fi

    if [ "$WS_HOST" == "0.0.0.0" ]; then
        CLIENT_WS_HOST="localhost"
    else
        CLIENT_WS_HOST="$WS_HOST"
    fi

    if [ "$DOC_HOST" == "0.0.0.0" ]; then
        CLIENT_DOC_HOST="localhost"
    else
        CLIENT_DOC_HOST="$DOC_HOST"
    fi
else
    echo -e "${YELLOW}Config file not found, using default values${NC}"
    CLIENT_API_HOST="localhost"
    API_PORT=8090
    CLIENT_WS_HOST="localhost"
    WS_PORT=8091
    CLIENT_DOC_HOST="localhost"
    DOC_PORT=8092
fi

# Check if the HTTP API server is running
http_url="http://${CLIENT_API_HOST}:${API_PORT}"
echo -n "HTTP API Server (${http_url}): "

# Try with any response code, not just 2xx
if curl --output /dev/null --silent --connect-timeout 2 "${http_url}"; then
    echo -e "${GREEN}RUNNING${NC}"

    # Check ping endpoint
    echo -n "API Ping Check: "
    if curl --output /dev/null --silent --connect-timeout 2 "${http_url}/api/ping"; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
    fi
else
    echo -e "${RED}NOT RUNNING${NC}"
fi

# Check if the WebSocket server is running
ws_url="http://${CLIENT_WS_HOST}:${WS_PORT}"
echo -n "WebSocket Server (${ws_url}): "

# Try to connect to the WebSocket port using netcat or curl
if command -v nc &> /dev/null; then
    if nc -z ${CLIENT_WS_HOST} ${WS_PORT} -w 2; then
        echo -e "${GREEN}RUNNING${NC}"
    else
        echo -e "${RED}NOT RUNNING${NC}"
    fi
elif curl --output /dev/null --silent --connect-timeout 2 "${ws_url}"; then
    echo -e "${GREEN}RUNNING${NC}"
else
    echo -e "${RED}NOT RUNNING${NC}"
fi

# Check if the Document Persistence API server is running
doc_url="http://${CLIENT_DOC_HOST}:${DOC_PORT}"
echo -n "Document Persistence API (${doc_url}): "

# Try with any response code, not just 2xx
if curl --output /dev/null --silent --connect-timeout 2 "${doc_url}"; then
    echo -e "${GREEN}RUNNING${NC}"

    # Check the document branch API
    echo -n "Document Branch API: "
    if curl --output /dev/null --silent --connect-timeout 2 "${doc_url}/api/branches/status"; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC} - This may cause 'Document branch not found' errors"
    fi
else
    echo -e "${RED}NOT RUNNING${NC} - This may cause 'Document branch not found' errors"
fi

# Print a helpful message about the correct HTML file to use
echo -e "\n${BLUE}Web Client:${NC}"
echo -e "For the updated interface with document branch fixes, use: ${GREEN}new-index.html${NC}"
echo -e "To access the editor: ${GREEN}http://localhost:8000/new-index.html${NC} (when using serve_with_cors.sh)"

# Check if the web client is already being served
echo -n "Web Client Server: "
if curl --output /dev/null --silent --connect-timeout 2 "http://localhost:8000"; then
    echo -e "${GREEN}RUNNING${NC}"
else
    echo -e "${YELLOW}NOT RUNNING${NC} - Run ./serve_with_cors.sh to start"
fi
