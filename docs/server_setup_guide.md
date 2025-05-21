# TeXSwarm Server Setup Guide

This guide provides detailed instructions for setting up and configuring the TeXSwarm server components.

## Server Components

TeXSwarm consists of three main server components:

1. **HTTP API Server** (Port 8090) - Handles REST API requests for document management
2. **WebSocket Server** (Port 8091) - Manages real-time collaborative editing sessions
3. **Document Persistence API** (Port 8092) - Handles document storage and persistence operations

## Installation

### Prerequisites

- Rust 2024 Edition or later (rustc 1.74+ recommended)
- Git 2.30+
- For the web frontend: Node.js 18+ and npm 9+

### Building from Source

```bash
# Clone the repository
git clone https://github.com/JonyBepary/TeXSwarm.git
cd TeXSwarm

# Build the project
cargo build --release
```

## Configuration

### Port Configuration

The default port settings are:

- HTTP API Server: Port 8090
- WebSocket Server: Port 8091
- Document Persistence API: Port 8092

To modify these settings, edit the `config.json` file:

```json
{
  "server": {
    "api_host": "0.0.0.0",
    "api_port": 8090,
    "ws_host": "0.0.0.0",
    "ws_port": 8091
  }
}
```

> **Note**: The Document Persistence API automatically uses port 8092 to avoid conflicts with the other services. If you need to change this port, you'll need to modify the source code.

### Network Interface Binding

By default, all server components bind to `0.0.0.0`, allowing connections from any network interface. If you want to restrict access to localhost only, change the configuration to:

```json
{
  "server": {
    "api_host": "127.0.0.1",
    "api_port": 8090,
    "ws_host": "127.0.0.1",
    "ws_port": 8091
  }
}
```

## Running the Server

### Development Mode

```bash
cargo run --bin p2p-latex-collab-server
```

### Production Mode

```bash
cargo run --release --bin p2p-latex-collab-server
```

## Troubleshooting

### Port Conflicts

If you encounter the error "Address already in use", it means another process is using one of the required ports. You have several options:

1. Change the port configuration in `config.json`
2. Stop the process using the conflicting port
3. Use the `--debug-config` flag to see which ports are being used:

```bash
cargo run --bin p2p-latex-collab-server -- --debug-config
```

### Checking Server Status

To verify that all server components are running correctly:

```bash
cd /path/to/TeXSwarm/web
./check_server_status.sh
```

## Web Client Configuration

The web client automatically connects to the API and WebSocket servers on ports 8090 and 8091 respectively. If you change these ports in the server configuration, you'll need to update the following constants in `/web/script.js`:

```javascript
const API_PORT = 8090;
const WS_PORT = 8091;
```

## Firewall Configuration

If you're running TeXSwarm behind a firewall, make sure to open the following ports:

- TCP port 8090 for HTTP API
- TCP port 8091 for WebSocket connections
- TCP port 8092 for Document Persistence API (internal use only)

## Load Balancing

If you're setting up a high-availability deployment, you can load balance the HTTP API (port 8090) and WebSocket servers (port 8091) across multiple instances. However, you'll need to ensure that:

1. The Document Persistence API is consistently accessible
2. Sessions for a specific document are directed to the same server instance

For more advanced deployment scenarios, please consult the [Advanced Deployment Guide](deployment_guide.md).
