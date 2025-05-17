#!/usr/bin/env bash

# Simple script to serve the web frontend demo

# Check if python3 is available
if command -v python3 &>/dev/null; then
    echo "Starting web server on http://localhost:8000"
    cd "$(dirname "$0")"
    python3 -m http.server 8000
# Check if python is available
elif command -v python &>/dev/null; then
    # Check Python version
    PYTHON_VERSION=$(python -c 'import sys; print(sys.version_info[0])')
    if [ "$PYTHON_VERSION" -eq 3 ]; then
        echo "Starting web server on http://localhost:8000"
        cd "$(dirname "$0")"
        python -m http.server 8000
    else
        echo "Starting web server on http://localhost:8000"
        cd "$(dirname "$0")"
        python -m SimpleHTTPServer 8000
    fi
else
    echo "Error: Python is not installed. Please install Python to run the web server."
    exit 1
fi
