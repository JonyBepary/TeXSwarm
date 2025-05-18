#!/bin/bash

# This script serves the new TeXSwarm web app using a simple HTTP server
# It's designed to be used during development for testing

# Define a different port to avoid conflicts (default was 8000)
PORT=8090

# Check if python3 is available
if command -v python3 &>/dev/null; then
    echo "Starting HTTP server with Python 3..."
    echo "Access the application at http://localhost:$PORT/new-index.html"
    cd "$(dirname "$0")" # Change to the script's directory
    python3 -m http.server $PORT
elif command -v python &>/dev/null; then
    # Check Python version
    python_version=$(python --version 2>&1)
    if [[ $python_version == *"Python 3"* ]]; then
        echo "Starting HTTP server with Python 3..."
        echo "Access the application at http://localhost:$PORT/new-index.html"
        cd "$(dirname "$0")" # Change to the script's directory
        python -m http.server $PORT
    else
        echo "Starting HTTP server with Python 2..."
        echo "Access the application at http://localhost:$PORT/new-index.html"
        cd "$(dirname "$0")" # Change to the script's directory
        python -m SimpleHTTPServer $PORT
    fi
else
    echo "Error: Python is not available. Please install Python or use another web server."
    exit 1
fi
