#!/bin/bash

# Simple script to serve the web app with CORS headers
# This version uses Python's built-in HTTP server instead of npm packages

echo "Starting TeXSwarm Web Server with Python..."

# Check if the backend server is running
echo "Checking if backend server is running at http://localhost:8080..."
if curl -s http://localhost:8080/api/ping > /dev/null; then
    echo "✅ Backend server is running!"
else
    echo "⚠️ Warning: Backend server does not appear to be running at http://localhost:8080."
    echo "Make sure the backend server is running with 'cargo run --bin p2p-latex-collab-server'"
    echo "Continuing anyway, but the web interface may not work correctly..."
fi

echo "Starting web server..."
echo "Access the web interface at: http://localhost:8000"
echo "Access the connection test at: http://localhost:8000/connectivity_test.html"
echo "Press Ctrl+C to stop the server"
echo ""

# Start a simple Python HTTP server
python3 -m http.server 8000 --bind 0.0.0.0
