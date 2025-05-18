#!/bin/bash

# Display host and port information for the server
echo "===== TeXSwarm Server Status ====="
echo "HTTP API: http://$(hostname -I | awk '{print $1}'):8080"
echo "WebSocket: ws://$(hostname -I | awk '{print $1}'):8081/ws"
echo "=================================="

# Try to ping the HTTP API
echo -n "Testing HTTP API... "
if curl -s --connect-timeout 5 http://localhost:8080/api/ping > /dev/null; then
    echo "Reachable"

    # If the ping endpoint is reachable, get the server info
    SERVER_INFO=$(curl -s http://localhost:8080/api/ping)
    echo "Server Info: $SERVER_INFO"
else
    echo "Not reachable"
fi

# Check if WebSocket port is open
echo -n "Testing WebSocket port... "
if nc -z localhost 8081 2>/dev/null; then
    echo "Port is open"
else
    echo "Port is not open"
fi

echo "=================================="
echo "If the tests fail, make sure the server is running with:"
echo "cargo run --bin p2p-latex-collab-server"
echo ""
echo "To access the TeXSwarm web interface from other devices:"
echo "1. Start the web server: ./web/serve_with_cors.sh"
echo "2. Visit http://$(hostname -I | awk '{print $1}'):8000 from another device"
echo "=================================="
