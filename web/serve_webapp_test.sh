#!/bin/bash
# Simple script to serve the webapp test page

# Set the port or use default 8000
PORT=${1:-8000}

echo "Starting web server on port $PORT"
echo "Open http://localhost:$PORT/webapp_test.html in your browser"

# Check if python3 is available
if command -v python3 > /dev/null; then
    python3 -m http.server $PORT
elif command -v python > /dev/null; then
    python -m http.server $PORT
else
    echo "Error: Python is not available. Please install Python or use another web server."
    exit 1
fi
