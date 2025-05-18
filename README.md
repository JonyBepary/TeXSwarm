# TeXSwarm: Decentralized LaTeX Collaboration Platform

A decentralized peer-to-peer alternative to Overleaf for collaborative LaTeX editing, enabling seamless document collaboration without centralized servers.

![TeXSwarm Logo](https://github.com/JonyBepary/TeXSwarm/raw/main/docs/images/logo.png)

## Overview

TeXSwarm enables real-time collaborative editing of LaTeX documents in a fully decentralized manner. Multiple users can work on the same document simultaneously, with changes automatically synchronized across all participants. The platform leverages conflict-free replicated data types (CRDTs) to ensure that all users see a consistent document state regardless of network conditions.

## Features

- **CRDT-based Collaborative Editing**: Uses diamond-types for conflict-free collaborative editing with automatic conflict resolution
- **P2P Networking**: Built on libp2p for decentralized peer-to-peer communication without requiring central servers
- **Git Integration**: Seamless synchronization with Git repositories for version control and backup
- **Modular Architecture**: Pluggable design allowing multiple frontends to connect to the same backend
- **API Layer**: Well-designed API and protocols for communication between components
- **Real-time Collaboration**: Multiple users can edit the same document simultaneously with changes visible in real-time
- **Offline Support**: Continue working without an internet connection and sync changes when connectivity is restored
- **User Presence**: See who is currently editing a document and their cursor positions
- **LaTeX Compilation**: Automatic compilation of LaTeX documents to preview changes

## Architecture

TeXSwarm is built with a modular architecture that separates concerns and allows for flexibility:

- **CRDT Engine**: The core of the system, managing document state and operations using diamond-types library
  - Handles the Conflict-free Replicated Data Type operations
  - Ensures eventual consistency across all peers
  - Manages document branches and their synchronization

- **Network Engine**: Manages peer discovery and communication in a decentralized network
  - Implements peer discovery via mDNS and Kademlia DHT
  - Handles document subscription and message routing
  - Ensures reliable message delivery even in challenging network conditions

- **Git Manager**: Provides Git integration for version control and collaboration
  - Syncs documents with Git repositories
  - Handles conflict resolution between CRDT operations and Git changes
  - Supports GitHub and other Git hosting services

- **API Layer**: Exposes HTTP and WebSocket interfaces for client applications
  - REST API for document management
  - WebSocket API for real-time collaboration
  - Comprehensive API documentation with examples

The following diagram illustrates the architecture:

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│                  │     │                  │     │                  │
│   Web Frontend   │     │  Mobile Frontend │     │ Desktop Frontend │
│                  │     │                  │     │                  │
└────────┬─────────┘     └────────┬─────────┘     └────────┬─────────┘
         │                        │                        │
         └────────────┬───────────┴────────────┬──────────┘
                      │                        │
               ┌──────▼─────────┐      ┌───────▼──────────┐
               │                │      │                  │
               │  HTTP API      │      │  WebSocket API   │
               │                │      │                  │
               └──────┬─────────┘      └───────┬──────────┘
                      │                        │
                      └────────────┬───────────┘
                                   │
         ┌────────────────────────┬┴─────────────────────────────┐
         │                        │                              │
┌────────▼─────────┐     ┌────────▼─────────┐          ┌─────────▼────────┐
│                  │     │                  │          │                  │
│   CRDT Engine    │◄────►   Network Engine │◄─────────►   Git Manager    │
│                  │     │                  │          │                  │
└──────────────────┘     └──────────────────┘          └──────────────────┘
```

## Document Synchronization

TeXSwarm implements a robust document synchronization mechanism that ensures all users see a consistent document state regardless of network conditions. The synchronization is based on the following components:

1. **CRDT Operations**: Document changes are encoded as CRDT operations to ensure eventual consistency
   - Insert operations add text at a specific position
   - Delete operations remove text from a range
   - Replace operations combine delete and insert
   - All operations include metadata like user ID and document ID

2. **Document Subscription**: Peers subscribe to document topics to receive updates
   - Topic-based publish/subscribe using libp2p gossipsub
   - Automatic discovery of peers editing the same document
   - Peers can join and leave documents dynamically

3. **Operation Broadcasting**: Changes are broadcast to all subscribed peers
   - Operations are encoded and broadcast to all peers in real-time
   - Multiple delivery mechanisms ensure operation delivery
   - Operations are applied to the local document state when received

4. **Branch Synchronization**: Document branches are synchronized between peers
   - Each peer maintains a branch of the document
   - CRDT algorithms ensure branches converge to the same state
   - Merge operations are handled automatically

Recent improvements to the document synchronization include:
- Fixed peer ID handling in the NetworkEngine to ensure proper peer identification
- Enhanced document subscription mechanism with better error handling and recovery
- Improved operation broadcasting to ensure delivery even in challenging network conditions
- Added document synchronization request capabilities to recover from missed operations

See [SYNCHRONIZATION_SOLUTION.md](SYNCHRONIZATION_SOLUTION.md) for detailed information on the document synchronization solution and how we addressed specific synchronization challenges.

## Getting Started

### Prerequisites

- Rust 2024 Edition or later (rustc 1.74+ recommended)
- Git 2.30+
- For the web frontend: Node.js 18+ and npm 9+

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/JonyBepary/TeXSwarm.git
cd TeXSwarm

# Build the project
cargo build --release

# Run the server
cargo run --release --bin server
```

#### Using Cargo

```bash
# Install from crates.io
cargo install texswarm

# Run the server
texswarm-server
```

#### Web Frontend

```bash
# Navigate to the web directory
cd TeXSwarm/web

# Install dependencies
npm install

# Start the development server
npm start
```

### Connecting Clients

Once the server is running, you can connect to it using any of the available clients:

- Web client: Navigate to `http://localhost:8080` in your browser
- API client: Connect to `http://localhost:8080/api`
- WebSocket client: Connect to `ws://localhost:8081`

### Configuration

TeXSwarm uses a configuration file located at `~/.config/texswarm/config.json`.
If this file doesn't exist, a default configuration will be created automatically on first run.

#### Configuration File Structure

```json
{
  "server": {
    "api_host": "127.0.0.1",
    "api_port": 8080,
    "ws_host": "127.0.0.1",
    "ws_port": 8081
  },
  "network": {
    "peer_id_seed": null,
    "bootstrap_nodes": [],
    "listen_addresses": ["/ip4/0.0.0.0/tcp/9000"],
    "external_addresses": [],
    "enable_mdns": true,
    "enable_kad": true
  },
  "git": {
    "repositories_path": "./repositories",
    "github_token": null,
    "github_username": null,
    "github_email": null
  },
  "storage": {
    "documents_path": "./documents",
    "max_document_size_mb": 50,
    "enable_autosave": true,
    "autosave_interval_seconds": 60
  }
}
```

#### Configuration Options

**Server Configuration**
- `api_host`: Host address for the HTTP API server
- `api_port`: Port for the HTTP API server
- `ws_host`: Host address for the WebSocket server
- `ws_port`: Port for the WebSocket server

**Network Configuration**
- `peer_id_seed`: Optional seed for generating a consistent peer ID
- `bootstrap_nodes`: List of nodes to connect to on startup
- `listen_addresses`: List of addresses to listen on for incoming connections
- `external_addresses`: List of external addresses to advertise
- `enable_mdns`: Enable mDNS peer discovery (local network)
- `enable_kad`: Enable Kademlia DHT for peer discovery

**Git Configuration**
- `repositories_path`: Path where Git repositories will be stored
- `github_token`: GitHub access token for repository access
- `github_username`: GitHub username for commits
- `github_email`: GitHub email for commits

**Storage Configuration**
- `documents_path`: Path where documents will be stored
- `max_document_size_mb`: Maximum document size in megabytes
- `enable_autosave`: Enable automatic saving of documents
- `autosave_interval_seconds`: Interval between autosaves

## API Documentation

### HTTP API

The HTTP API is available at `http://{api_host}:{api_port}` and follows RESTful conventions. All responses are in JSON format.

#### Authentication

Most endpoints require authentication using a JWT token. To authenticate, include the token in the `Authorization` header:

```
Authorization: Bearer <your-token>
```

#### Document Endpoints

| Endpoint | Method | Description | Request Body | Response |
|----------|--------|-------------|-------------|----------|
| `/documents` | GET | List all documents | - | Array of document metadata |
| `/documents` | POST | Create a new document | `{ "title": "string", "content": "string" }` | Document metadata |
| `/documents/{id}` | GET | Get document metadata | - | Document metadata |
| `/documents/{id}/content` | GET | Get document content | - | Document content |
| `/documents/{id}/content` | PUT | Update document content | Raw document content | Success status |
| `/documents/{id}/operations` | POST | Apply operation to document | Operation object | Success status |
| `/documents/{id}/sync` | POST | Synchronize with Git repository | - | Sync status |

#### User Endpoints

| Endpoint | Method | Description | Request Body | Response |
|----------|--------|-------------|-------------|----------|
| `/users/register` | POST | Register a new user | User registration details | User metadata with token |
| `/users/login` | POST | Login | Login credentials | Authentication token |
| `/users/profile` | GET | Get user profile | - | User profile data |
| `/users/profile` | PUT | Update user profile | Profile update details | Updated profile |

For detailed information about request and response formats, see the [API Protocol Documentation](docs/api_protocol.md).

### WebSocket API

The WebSocket API is available at `ws://{ws_host}:{ws_port}` and enables real-time collaboration and notifications.

#### Connection

To connect to the WebSocket API, establish a WebSocket connection to the server URL. Authentication is performed by sending an authentication message immediately after connection:

```json
{
  "type": "authenticate",
  "token": "your-jwt-token"
}
```

#### Message Types

Messages sent and received through the WebSocket connection follow a common format:

```json
{
  "type": "string",
  "data": {
    // Message-specific data
  }
}
```

Common message types include:

| Type | Direction | Description | Data Structure |
|------|-----------|-------------|----------------|
| `operation` | Client → Server | Document operation | CRDT operation details |
| `presence` | Client → Server | User presence update | Cursor position, selection |
| `document_update` | Server → Client | Document updated | Updated document content |
| `presence_update` | Server → Client | User presence changed | User ID, cursor position |
| `error` | Server → Client | Error occurred | Error code and message |

For detailed information about WebSocket message formats, see [`src/api/protocol.rs`](src/api/protocol.rs).

## Development

### Project Structure

```
TeXSwarm/
├── src/
│   ├── api/             # API layer implementation
│   │   ├── http.rs      # HTTP API handlers
│   │   ├── protocol.rs  # API protocol definitions
│   │   ├── server.rs    # Server implementation
│   │   └── websocket.rs # WebSocket API handlers
│   ├── bin/             # Executable binaries
│   │   ├── server.rs    # Main server binary
│   │   └── ...          # Test and utility binaries
│   ├── crdt/            # CRDT implementation
│   │   ├── document.rs  # Document model
│   │   ├── engine.rs    # CRDT engine
│   │   └── operations.rs # CRDT operations
│   ├── git/             # Git integration
│   │   ├── manager.rs   # Git manager
│   │   └── repository.rs # Git repository
│   ├── network/         # P2P networking
│   │   ├── engine.rs    # Network engine
│   │   ├── peer.rs      # Peer management
│   │   └── protocol.rs  # Network protocol
│   ├── utils/           # Utility functions
│   │   ├── config.rs    # Configuration
│   │   └── errors.rs    # Error handling
│   └── lib.rs           # Library entry point
├── web/                 # Web frontend
│   ├── src/             # Frontend source code
│   ├── public/          # Static assets
│   └── package.json     # Frontend dependencies
├── docs/                # Documentation
├── tests/               # Integration tests
└── README.md            # This file
```

### Building and Testing

#### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test -- test_name

# Run tests with logging
RUST_LOG=debug cargo test
```

### Contributing

Contributions to TeXSwarm are welcome! Here's how to get started:

1. Fork the repository
2. Create a new branch for your feature
3. Add your changes
4. Run tests to ensure everything works
5. Submit a pull request

Please follow the [Rust Code Style Guidelines](https://rustc-dev-guide.rust-lang.org/conventions.html) for your code.

## Troubleshooting

### Common Issues

#### Document Synchronization Issues

If you're experiencing document synchronization issues, check the following:

1. Ensure all instances are connected to the network
2. Verify that the document exists in all instances
3. Check that document subscription is working correctly
4. See [DOCUMENT_SYNC_ISSUES.md](DOCUMENT_SYNC_ISSUES.md) for detailed diagnostics

#### Network Connectivity Issues

If peers cannot discover each other:

1. Make sure mDNS is enabled in the configuration
2. Check if the bootstrap nodes are reachable
3. Verify that your network allows UDP traffic

#### Git Integration Issues

If Git synchronization is not working:

1. Check the Git credentials in the configuration
2. Ensure the repository exists and is accessible
3. Verify that the GitHub token has the correct permissions

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Recent Network Fixes

We've implemented comprehensive fixes for document synchronization issues:

1. **Complete Network Layer Implementation**:
   - Added full P2P networking with libp2p
   - Implemented gossipsub for topic-based messaging
   - Created request-response protocols for direct peer communication

2. **Robust Document Synchronization**:
   - Fixed peer ID handling and document subscription
   - Implemented proper document content synchronization
   - Added reliable operation broadcasting between peers

3. **Migration Path from Mock to Real Implementation**:
   - Created service wrapper for smooth transition
   - Added migration script for testing real implementation
   - Maintained backward compatibility with existing code

For migration instructions, see [Network Implementation Plan](docs/network_implementation_plan.md).

## Acknowledgments

- [diamond-types](https://github.com/josephg/diamond-types) for CRDT implementation
- [libp2p](https://github.com/libp2p/rust-libp2p) for peer-to-peer networking
- [LaTeX](https://www.latex-project.org/) for document typesetting
- Me who have helped with development
<!-- - All contributors who have helped with development -->
