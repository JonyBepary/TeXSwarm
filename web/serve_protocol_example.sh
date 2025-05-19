#!/bin/bash
# A simple script to serve the WebSocket protocol example

echo "Starting server for WebSocket Protocol Example on http://localhost:8000"
echo "Press Ctrl+C to stop"

cd "$(dirname "$0")"
python3 -m http.server 8000
