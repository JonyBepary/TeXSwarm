#!/bin/bash
# Comprehensive TeXSwarm Web App Testing Tool
# This script helps test both the HTTP API and WebSocket connectivity

# Define colors for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_HOST=${API_HOST:-"localhost"}
API_PORT=${API_PORT:-8080}
WS_HOST=${WS_HOST:-"localhost"}
WS_PORT=${WS_PORT:-8081}

# Print header
echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}TeXSwarm Web App Connectivity Tester${NC}"
echo -e "${BLUE}====================================${NC}"
echo ""
echo -e "Testing with configuration:"
echo -e "  HTTP API: ${YELLOW}http://$API_HOST:$API_PORT${NC}"
echo -e "  WebSocket: ${YELLOW}ws://$WS_HOST:$WS_PORT${NC}"
echo ""

# Test HTTP API ping endpoint
echo -e "${BLUE}[1/5] Testing HTTP API ping endpoint...${NC}"
PING_RESULT=$(curl -s -o /dev/null -w "%{http_code}" http://$API_HOST:$API_PORT/api/ping)
if [ "$PING_RESULT" = "200" ]; then
    echo -e "  ${GREEN}✓ API ping endpoint is working (HTTP 200)${NC}"
else
    echo -e "  ${RED}✗ API ping endpoint is not working (HTTP $PING_RESULT)${NC}"
    echo -e "  ${YELLOW}ℹ Try checking if the server is running and accessible${NC}"
fi

# Test user registration
echo -e "\n${BLUE}[2/5] Testing user registration...${NC}"
USER_RESULT=$(curl -s -X POST -H "Content-Type: application/json" -d '{"name":"TestUser"}' http://$API_HOST:$API_PORT/api/users)
if [[ $USER_RESULT == *"id"* && $USER_RESULT == *"name"* ]]; then
    echo -e "  ${GREEN}✓ User registration is working${NC}"
    USER_ID=$(echo $USER_RESULT | sed -E 's/.*"id":"([^"]+)".*/\1/')
    echo -e "  ${YELLOW}ℹ Generated user ID: $USER_ID${NC}"
else
    echo -e "  ${RED}✗ User registration failed${NC}"
    echo -e "  ${YELLOW}ℹ Response: $USER_RESULT${NC}"
fi

# Test document listing
echo -e "\n${BLUE}[3/5] Testing document listing...${NC}"
DOCS_RESULT=$(curl -s -H "X-User-ID: $USER_ID" http://$API_HOST:$API_PORT/api/documents)
if [[ $DOCS_RESULT == *"documents"* || $DOCS_RESULT == "{}" ]]; then
    echo -e "  ${GREEN}✓ Document listing is working${NC}"
    echo -e "  ${YELLOW}ℹ Response: $DOCS_RESULT${NC}"
else
    echo -e "  ${RED}✗ Document listing failed${NC}"
    echo -e "  ${YELLOW}ℹ Response: $DOCS_RESULT${NC}"
fi

# Test document creation
echo -e "\n${BLUE}[4/5] Testing document creation...${NC}"
# Create a temporary JSON file for the request body to avoid shell escaping issues
cat > /tmp/doc_request.json << EOF
{
  "title": "Test Document",
  "owner_id": "$USER_ID"
}
EOF

DOC_RESULT=$(curl -s -X POST -H "Content-Type: application/json" -H "X-User-ID: $USER_ID" -d @/tmp/doc_request.json http://$API_HOST:$API_PORT/api/documents)
if [[ $DOC_RESULT == *"document_id"* || $DOC_RESULT == *"id"* ]]; then
    echo -e "  ${GREEN}✓ Document creation is working${NC}"
    echo -e "  ${YELLOW}ℹ Response: $DOC_RESULT${NC}"
else
    echo -e "  ${RED}✗ Document creation failed${NC}"
    echo -e "  ${YELLOW}ℹ Response: $DOC_RESULT${NC}"
fi

# Test WebSocket connectivity
echo -e "\n${BLUE}[5/5] Testing WebSocket connectivity...${NC}"
echo -e "  ${YELLOW}ℹ Connecting to WebSocket...${NC}"

# Use websocat if available, otherwise fallback to a simple JavaScript test
if command -v websocat >/dev/null 2>&1; then
    echo -e "  ${YELLOW}ℹ Using websocat for testing...${NC}"
    echo "{\"type\":\"Authentication\",\"payload\":{\"user_id\":\"$USER_ID\"}}" | timeout 5 websocat "ws://$WS_HOST:$WS_PORT/ws" && WEBSOCKET_WORKING=true || WEBSOCKET_WORKING=false

    if [ "$WEBSOCKET_WORKING" = true ]; then
        echo -e "  ${GREEN}✓ WebSocket connection is working${NC}"
    else
        echo -e "  ${RED}✗ WebSocket connection failed${NC}"
    fi
else
    echo -e "  ${YELLOW}ℹ websocat not found, creating a JavaScript test...${NC}"

    # Create a temporary JavaScript test file
    cat > test_websocket.js << EOF
const WebSocket = require('ws');
const ws = new WebSocket('ws://$WS_HOST:$WS_PORT/ws');

ws.on('open', function open() {
  console.log('Connected to WebSocket');
  ws.send(JSON.stringify({
    type: 'Authentication',
    payload: {
      user_id: '$USER_ID'
    }
  }));

  // Close after a short delay
  setTimeout(() => {
    console.log('Test completed successfully');
    ws.close();
    process.exit(0);
  }, 2000);
});

ws.on('message', function incoming(data) {
  console.log('Received: %s', data);
});

ws.on('error', function error(err) {
  console.error('WebSocket error: %s', err);
  process.exit(1);
});

// Timeout after 5 seconds
setTimeout(() => {
  console.error('Connection timeout');
  process.exit(1);
}, 5000);
EOF

    # Check if Node.js is available
    if command -v node >/dev/null 2>&1; then
        node test_websocket.js && WEBSOCKET_WORKING=true || WEBSOCKET_WORKING=false

        if [ "$WEBSOCKET_WORKING" = true ]; then
            echo -e "  ${GREEN}✓ WebSocket connection is working${NC}"
        else
            echo -e "  ${RED}✗ WebSocket connection failed${NC}"
        fi

        rm test_websocket.js
    else
        echo -e "  ${RED}✗ Cannot test WebSocket - neither websocat nor Node.js is available${NC}"
        echo -e "  ${YELLOW}ℹ To test manually, open a browser console and run:${NC}"
        echo -e "  ${YELLOW}    const ws = new WebSocket('ws://$WS_HOST:$WS_PORT/ws');${NC}"
        echo -e "  ${YELLOW}    ws.onopen = () => console.log('Connected');${NC}"
        echo -e "  ${YELLOW}    ws.onerror = (e) => console.error('Error', e);${NC}"
    fi
fi

# Final summary
echo ""
echo -e "${BLUE}======== Test Summary ========${NC}"
if [ "$PING_RESULT" = "200" ]; then
    echo -e "${GREEN}✓ HTTP API is accessible${NC}"
else
    echo -e "${RED}✗ HTTP API is not accessible${NC}"
fi

if [ "$WEBSOCKET_WORKING" = true ]; then
    echo -e "${GREEN}✓ WebSocket is accessible${NC}"
else
    echo -e "${RED}✗ WebSocket is not accessible${NC}"
fi
echo ""

# Try to open the web app in the local browser
echo -e "${YELLOW}ℹ To open the web app in your browser, visit:${NC}"
echo -e "${BLUE}http://$API_HOST/web/index.html${NC} (if served by your server)"
echo -e "Or run this command to serve it locally:"
echo -e "${BLUE}cd ../web && python3 -m http.server 8000${NC}"
echo ""
echo -e "${BLUE}====================================${NC}"
