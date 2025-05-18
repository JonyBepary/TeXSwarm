#!/bin/bash

# This script serves the WebSocket debug page

echo "Starting simple HTTP server for WebSocket debug page..."
echo "You can access the page at: http://localhost:8000/websocket_debug.html"
echo "Press Ctrl+C to stop the server"

# Change to the web directory and start a simple Python HTTP server
cd "$(dirname "$0")" && python3 -m http.server
