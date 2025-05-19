#!/bin/bash

# This script serves the WebSocket protocol test page

# Define port
PORT=8095

# Check if python3 is available
if command -v python3 &>/dev/null; then
    echo "Starting HTTP server with Python 3..."
    echo "Access the test page at http://localhost:$PORT/websocket-test.html"
    cd "$(dirname "$0")" # Change to the script's directory
    python3 -m http.server $PORT
elif command -v python &>/dev/null; then
    # Check Python version
    python_version=$(python --version 2>&1)
    if [[ $python_version == *"Python 3"* ]]; then
        echo "Starting HTTP server with Python 3..."
        echo "Access the test page at http://localhost:$PORT/websocket-test.html"
        cd "$(dirname "$0")" # Change to the script's directory
        python -m http.server $PORT
    else
        echo "Starting HTTP server with Python 2..."
        echo "Access the test page at http://localhost:$PORT/websocket-test.html"
        cd "$(dirname "$0")" # Change to the script's directory
        python -m SimpleHTTPServer $PORT
    fi
else
    echo "Error: Python is not installed. Please install Python to use this script."
    exit 1
fi
