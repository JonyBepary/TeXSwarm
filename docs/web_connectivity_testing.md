# TeXSwarm Web Connectivity Guide

This guide helps you test and troubleshoot connections between the TeXSwarm web interface and the backend server.

## Prerequisites

1. The TeXSwarm server is running using:
   ```bash
   cd /home/jony/rust-pract/TeXSwarm
   cargo run --bin p2p-latex-collab-server
   ```

2. The web interface is served with CORS support:
   ```bash
   cd /home/jony/rust-pract/TeXSwarm/web
   ./serve_with_cors.sh
   ```

## Quick Test

Run the connection test script:
```bash
./web/test_connection.sh
```

This will show you:
- The HTTP and WebSocket endpoints
- Whether the HTTP API is reachable
- Whether the WebSocket port is open
- Your local IP address for accessing from other devices

## Testing with the Browser

1. Open the connection test page: http://localhost:8000/connection_test.html
2. The page will automatically test connections to both HTTP API and WebSocket
3. Look for success messages in green or error messages in red

## Common Issues and Solutions

### Problem: Server only listens on localhost (127.0.0.1)

**Symptoms**:
- The server works from the local machine but not from other devices
- Logs show the server binding to 127.0.0.1 instead of 0.0.0.0

**Solution**:
We've modified the server to explicitly bind to 0.0.0.0 (all interfaces).
Make sure you're running the latest version of the code.

### Problem: CORS errors in browser console

**Symptoms**:
- "Access-Control-Allow-Origin" errors in browser console
- HTTP requests fail but WebSocket works

**Solution**:
Always use the provided `serve_with_cors.sh` script to serve the web interface.
This adds the necessary CORS headers for API requests.

### Problem: WebSocket fails to connect

**Symptoms**:
- "WebSocket connection failed" messages
- The HTTP API works but WebSocket doesn't

**Solution**:
- Check that the WebSocket server is running (port 8081 is open)
- Verify you're using the correct WebSocket path (/ws)
- Check for any firewalls blocking WebSocket traffic

## Testing from Other Devices

To test from other devices on the same network:

1. Find your machine's IP address (shown in test_connection.sh output)
2. Access the web interface at: http://YOUR_IP_ADDRESS:8000
3. The connection test page will be at: http://YOUR_IP_ADDRESS:8000/connection_test.html

Note: Make sure your firewall allows connections to ports 8000, 8080, and 8081.
