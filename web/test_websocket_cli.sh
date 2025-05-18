#!/bin/bash

# Test script to check WebSocket connectivity

# Default values
HOST=${1:-localhost}
PORT=${2:-8081}

echo "Testing WebSocket connectivity on $HOST:$PORT..."

# Check if websocat is installed
if ! command -v websocat &> /dev/null; then
    echo "The 'websocat' tool is not installed. Please install it to test WebSocket connections."
    echo "On Ubuntu/Debian: sudo apt install websocat"
    echo "On macOS: brew install websocat"
    exit 1
fi

# Try to connect to the WebSocket server
echo "Attempting WebSocket connection to ws://$HOST:$PORT/ws..."
echo "{\"type\":\"Authentication\",\"user_id\":\"test_user\",\"token\":null}" | websocat "ws://$HOST:$PORT/ws" --no-close -n --ping-interval 1

# Check if the connection was successful
if [ $? -eq 0 ]; then
    echo "✅ WebSocket connection successful!"
else
    echo "❌ WebSocket connection failed!"
fi
