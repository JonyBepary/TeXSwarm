# TeXSwarm Server Connectivity Testing

This document outlines the methods and tools for testing the connectivity of the TeXSwarm server components.

## Server Components

The TeXSwarm server has three main components:

1. **HTTP API Server** - Running on port 8090 (default)
2. **WebSocket Server** - Running on port 8091 (default)
3. **Document Persistence API** - Running on port 8092 (default)

All servers bind to all network interfaces (`0.0.0.0`) allowing them to accept connections from any source, not just localhost.

## Testing Tools

### Command-line Tests

1. **Basic Ping Test**:
   ```bash
   ./test_ping.sh [host] [port]
   ```
   Tests the HTTP API ping endpoint to verify the server is responsive.

2. **Comprehensive Connectivity Test**:
   ```bash
   ./test_server_connectivity.sh [host] [http_port] [ws_port] [persistence_port]
   ```
   Tests all server components (HTTP API, WebSocket server, and Document Persistence API) to verify connectivity.

### Web-based Tests

1. **Ping Test Page**:
   ```bash
   ./serve_ping_test.sh
   ```
   Serves a web page at http://localhost:8000/ping_test.html to test the HTTP API ping endpoint.

2. **Full Connectivity Test Page**:
   ```bash
   ./serve_connectivity_test.sh
   ```
   Serves a web page at http://localhost:8000/connectivity_test.html that can test both HTTP API and WebSocket connectivity.

## Test Procedures

1. Start the TeXSwarm server:
   ```bash
   cd /path/to/TeXSwarm
   cargo run --bin p2p-latex-collab-server
   ```

2. Run the test(s) of your choice from the `web` directory:
   ```bash
   cd /path/to/TeXSwarm/web
   ./test_server_connectivity.sh
   ```

3. For web-based tests, start the test server and open the page in your browser:
   ```bash
   cd /path/to/TeXSwarm/web
   ./serve_connectivity_test.sh
   ```

## Troubleshooting

If tests fail:

1. Verify that the server is running
2. Check if the ports are accessible (not blocked by firewall)
3. Ensure the server is binding to the correct address (should be `0.0.0.0`)
4. Check logs for any startup errors

## Network Issues Fixed

The previous issue where the server was only binding to `127.0.0.1` (localhost) has been fixed in the server implementation. The server now:

1. Explicitly sets the binding addresses to `0.0.0.0`
2. Provides more detailed logging of socket addresses
3. Successfully binds to all network interfaces

## Additional Improvements

1. Added a `/api/ping` endpoint for easier connectivity testing
2. Fixed compiler warnings about unused fields
3. Created comprehensive testing tools for both HTTP API and WebSocket
