#!/bin/bash
# Test script to check if the server's user API endpoint is working

# Default values
HOST=${1:-localhost}
PORT=${2:-8080}

echo "Testing user API on $HOST:$PORT..."

# Use curl to test the user API
echo "Attempting to create a test user..."
RESPONSE=$(curl -s -X POST "http://$HOST:$PORT/api/users" \
    -H "Content-Type: application/json" \
    -d '{"name": "test_user"}')

echo "Response: $RESPONSE"

if [ $? -eq 0 ]; then
    if [[ "$RESPONSE" == *"id"* ]]; then
        echo "✅ User API test passed! The server responded with a user ID."
    else
        echo "❌ User API test failed! The server responded but with unexpected content."
    fi
else
    echo "❌ User API test failed! Could not connect to the server at $HOST:$PORT."
fi
