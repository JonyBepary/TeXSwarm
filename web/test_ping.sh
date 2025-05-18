#!/bin/bash
# Simple script to test the ping endpoint of the TeXSwarm server

# Default values
HOST=${1:-localhost}
PORT=${2:-8080}

echo "Testing ping endpoint on $HOST:$PORT..."

# Use curl to get the ping response
RESPONSE=$(curl -s "http://$HOST:$PORT/api/ping")

# Check if the request was successful
if [ $? -eq 0 ]; then
    echo "Ping response: $RESPONSE"

    # Check if the response contains "status":"ok"
    if echo "$RESPONSE" | grep -q '"status":"ok"'; then
        echo "✅ Ping test passed! The server is running correctly."
    else
        echo "❌ Ping test failed! The server responded but with unexpected content."
    fi
else
    echo "❌ Ping test failed! Could not connect to the server at $HOST:$PORT."
fi
