#!/bin/bash

# This script serves the connectivity_test.html file for testing server connections

echo "Starting simple HTTP server for connectivity_test.html..."
echo "You can access the test page at: http://localhost:8000/connectivity_test.html"
echo "Press Ctrl+C to stop the server"

# Change to the web directory and start a simple Python HTTP server
cd "$(dirname "$0")" && python3 -m http.server
