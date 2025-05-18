# TeXSwarm Web Connectivity Troubleshooting

This document provides guidance on troubleshooting connectivity issues between the TeXSwarm web frontend and the backend server.

## Server Configuration

The server uses the following default configuration:
- HTTP API Server: `http://localhost:8080`
- WebSocket Server: `ws://localhost:8081/ws`

These settings can be found in the following files:
1. Backend config: `/home/jony/rust-pract/TeXSwarm/src/utils/config.rs`
2. Frontend config: `/home/jony/rust-pract/TeXSwarm/web/script.js`

## Testing Connectivity

### Step 1: Check if the server is running

Run the provided script to check if both the HTTP and WebSocket servers are active:

```bash
cd /home/jony/rust-pract/TeXSwarm/web
./check_server_status.sh
```

If the servers are not running, start them with:

```bash
cd /home/jony/rust-pract/TeXSwarm
cargo run --bin server
```

### Step 2: Use the connection test tool

We've created a special connection test tool to diagnose specific connectivity issues:

```bash
cd /home/jony/rust-pract/TeXSwarm/web
./serve_with_cors.sh
```

Then open `http://localhost:8000/connection_test.html` in your browser.

### Step 3: Check browser console for errors

Open your browser's developer tools (F12 or right-click → Inspect) and check the Console tab for any error messages.

Common errors include:
- `Failed to fetch` - The HTTP server is unreachable
- `WebSocket connection failed` - The WebSocket server is unreachable
- `CORS error` - Cross-Origin Resource Sharing issues

## Common Issues and Solutions

### 1. Connection Refused

**Symptoms:**
- "Connection refused" errors
- Server shows as "NOT RUNNING" in the status check

**Potential causes:**
- The server isn't running
- Server is listening on a different port
- Firewall is blocking the connection

**Solutions:**
- Start the server with `cargo run --bin server`
- Check if the server started successfully
- Verify port configuration in both frontend and backend

### 2. CORS (Cross-Origin Resource Sharing) Issues

**Symptoms:**
- HTTP requests fail with CORS errors in the browser console
- API calls work in Postman but not in the browser

**Solutions:**
- Use the provided `serve_with_cors.sh` script which enables CORS headers
- Ensure both the web server and backend run on the same origin (host+port)
- If hosting on different domains, update the backend CORS configuration

### 3. WebSocket Connection Issues

**Symptoms:**
- WebSocket connections fail but HTTP works
- Disconnection status showing in the web app

**Solutions:**
- Ensure the WebSocket server is running on port 8081
- Check browser console for specific WebSocket errors
- Try connecting via IP address (127.0.0.1) instead of hostname
- Check firewall settings for WebSocket traffic

### 4. Network Configuration

**Symptoms:**
- Can't connect from other machines on the network
- Works locally but not remotely

**Solutions:**
- Make sure the server is listening on all interfaces (0.0.0.0) instead of just localhost
- Configure network/firewall to allow traffic on ports 8080 and 8081
- Update API_HOST and WS_HOST in script.js to use the server's network IP address

## Advanced Debugging

### Testing WebSocket with curl

```bash
curl --include \
     --no-buffer \
     --header "Connection: Upgrade" \
     --header "Upgrade: websocket" \
     --header "Host: localhost:8081" \
     --header "Origin: http://localhost:8000" \
     --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     --header "Sec-WebSocket-Version: 13" \
     http://localhost:8081/ws
```

### Testing HTTP API with curl

```bash
curl -v http://localhost:8080/api/ping
```

If the server doesn't have a ping endpoint, try another known endpoint:

```bash
curl -v http://localhost:8080/
```

## Need Further Help?

If you've tried all these steps and still have connectivity issues:

1. Check the server logs for any errors
2. Verify that your network allows the required connections
3. Check for any system-level restrictions (SELinux, AppArmor, etc.)
4. Try restarting both the client and server
5. Update to the latest version of the software
