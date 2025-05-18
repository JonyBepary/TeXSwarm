#!/bin/bash
# Comprehensive script to test server connectivity for both HTTP API and WebSocket

# Default values
HOST=${1:-localhost}
HTTP_PORT=${2:-8080}
WS_PORT=${3:-8081}

echo "=====================================================
 TeXSwarm Server Connectivity Test
====================================================="

# ------------------------------------------------------
# 1. HTTP API Ping Test
# ------------------------------------------------------
echo -e "\n📡 Testing HTTP API on $HOST:$HTTP_PORT..."

# Use curl to get the ping response
RESPONSE=$(curl -s "http://$HOST:$HTTP_PORT/api/ping")

# Check if the request was successful
if [ $? -eq 0 ]; then
    echo "Ping response: $RESPONSE"

    # Check if the response contains "status":"ok"
    if echo "$RESPONSE" | grep -q '"status":"ok"'; then
        echo "✅ HTTP API test passed! The server is running correctly."
    else
        echo "❌ HTTP API test failed! The server responded but with unexpected content."
    fi
else
    echo "❌ HTTP API test failed! Could not connect to the server at $HOST:$HTTP_PORT."
fi

# ------------------------------------------------------
# 2. WebSocket Server Test
# ------------------------------------------------------
echo -e "\n📡 Testing WebSocket Server on $HOST:$WS_PORT..."

# Using websocat if available, otherwise fall back to a simple connection test
if command -v websocat &> /dev/null; then
    # Try to connect to the WebSocket server with a timeout
    WS_RESULT=$(timeout 3 websocat -n1 ws://$HOST:$WS_PORT 2>&1 || echo "Connection failed")

    if [[ "$WS_RESULT" == *"Connection failed"* ]]; then
        echo "❌ WebSocket test failed! Could not connect to the WebSocket server at $HOST:$WS_PORT."
    else
        echo "✅ WebSocket test passed! Connected to the WebSocket server successfully."
    fi
else
    # Fallback: Just check if the port is open
    nc -z -w 3 $HOST $WS_PORT
    if [ $? -eq 0 ]; then
        echo "✅ WebSocket port is open. Connection possible."
    else
        echo "❌ WebSocket test failed! Port $WS_PORT is not reachable on $HOST."
    fi
fi

# ------------------------------------------------------
# 3. Summary
# ------------------------------------------------------
echo -e "\n📊 Connectivity Test Summary:"
echo "-----------------------------------------------------"
echo "HTTP API (Port $HTTP_PORT): $(if echo "$RESPONSE" | grep -q '"status":"ok"'; then echo "✅ WORKING"; else echo "❌ FAILED"; fi)"
echo "WebSocket (Port $WS_PORT): $(if nc -z -w 1 $HOST $WS_PORT; then echo "✅ ACCESSIBLE"; else echo "❌ FAILED"; fi)"
echo "-----------------------------------------------------"

echo -e "\nTest completed on $(date)"
