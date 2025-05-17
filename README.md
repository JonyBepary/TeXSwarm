# TeXSwarm

![TeXSwarm Logo](https://via.placeholder.com/150?text=TeXSwarm)

A decentralized peer-to-peer alternative to Overleaf for collaborative LaTeX editing. TeXSwarm enables multiple users to work on LaTeX documents simultaneously with real-time synchronization.

## Features

- **CRDT-based Collaborative Editing**: Uses diamond-types for conflict-free collaborative editing
- **P2P Networking**: Built on libp2p for decentralized peer-to-peer communication
- **Git Integration**: Seamless synchronization with Git repositories
- **Modular Architecture**: Pluggable design allowing multiple frontends to connect
- **API Layer**: Well-designed API and protocols for communication
- **Real-time Collaboration**: Edit documents simultaneously with multiple users
- **No Central Server**: Fully decentralized architecture eliminates single points of failure
- **Versioning**: Complete document history with branch and merge support

## Architecture

The system consists of several core components:

- **CRDT Engine**: Manages document state and operations using diamond-types
- **Network Engine**: Handles P2P discovery and communication via libp2p
- **Git Manager**: Provides Git repository integration for external synchronization
- **API Layer**: Exposes HTTP and WebSocket interfaces for frontends

### Component Interactions

```
┌─────────────┐     ┌─────────────┐
│  API Layer  │◄───►│  Frontends  │
└──────┬──────┘     └─────────────┘
       │
┌──────▼──────┐     ┌─────────────┐
│ CRDT Engine │◄───►│    Git      │
└──────┬──────┘     │   Manager   │
       │            └─────────────┘
┌──────▼──────┐     ┌─────────────┐
│  Network    │◄───►│  Other      │
│  Engine     │     │  Peers      │
└─────────────┘     └─────────────┘
```

## Document Synchronization

The project uses a robust document synchronization mechanism based on the following components:

1. **CRDT Operations**: Document changes are encoded as CRDT operations to ensure eventual consistency
2. **Document Subscription**: Peers subscribe to document topics to receive updates
3. **Operation Broadcasting**: Changes are broadcast to all subscribed peers
4. **Branch Synchronization**: Document branches are synchronized between peers

Recent improvements to the document synchronization include:
- Fixed peer ID handling in the NetworkEngine
- Enhanced document subscription mechanism
- Improved operation broadcasting to ensure delivery
- Added document synchronization request capabilities

See [SYNCHRONIZATION_SOLUTION.md](SYNCHRONIZATION_SOLUTION.md) for detailed information on the document synchronization solution.

## Technical Implementation

### CRDT Implementation

TeXSwarm uses the diamond-types CRDT library for handling concurrent document edits. Diamond-types offers:

- Efficient list operations for text documents
- Custom data type support
- Compact binary encoding
- Optimized merge operations
- Causality tracking for conflict resolution

### Network Protocol

The network protocol includes these key components:

- Document discovery and subscription
- Operation broadcasting with topic-based PubSub
- Direct peer-to-peer synchronization requests
- Mesh network topology for resilience

### Document Operations

Three main operation types are supported:
- **Insert**: Add text at a specific position
- **Delete**: Remove text from a range
- **Replace**: Combined delete and insert operations

## Getting Started

### Prerequisites

- Rust 2024 Edition or later
- Git
- Node.js 18+ (for web frontend)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/JonyBepary/TeXSwarm.git
cd TeXSwarm

# Build the project
cargo build --release

# Run the server
cargo run --release --bin server
```

### Development Setup

```bash
# Run in development mode
cargo run --bin server

# Run the web frontend
cd web
npm install
npm start

# Run tests
cargo test
```

### Configuration

The application uses a configuration file located at `~/.config/texswarm/config.json`.
If this file doesn't exist, a default configuration will be created automatically.

Example configuration:

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

## API Documentation

### HTTP API

The HTTP API is available at `http://{api_host}:{api_port}`.

#### Endpoints

- `POST /documents`: Create a new document
- `GET /documents`: List all documents
- `GET /documents/{id}`: Get document metadata
- `GET /documents/{id}/content`: Get document content
- `POST /documents/{id}/insert`: Insert text into a document
- `POST /documents/{id}/delete`: Delete text from a document
- `POST /documents/{id}/replace`: Replace text in a document
- `POST /documents/{id}/sync`: Synchronize with Git repository
- `GET /peers`: List connected peers
- `GET /status`: Get node status

### WebSocket API

The WebSocket API is available at `ws://{ws_host}:{ws_port}`.

Messages use the protocol defined in `src/api/protocol.rs`. The WebSocket API supports real-time collaboration with the following message types:

- Document subscription
- Operation broadcasting
- Presence updates
- Document synchronization
- Error handling

## Known Issues and Solutions

### Document Synchronization Issues

Our recent testing has identified and fixed several document synchronization issues:

1. **Missing Method Implementation**: Fixed the `get_local_peer_id()` method in NetworkEngine.
2. **Document Branch Propagation**: Added explicit document branch sharing between peers.
3. **Subscription Handling**: Enhanced document subscription to include content synchronization.
4. **Network Mock Limitations**: Created better workarounds for the mock network implementation.

For a complete description of the issues and fixes, see the [DOCUMENT_SYNC_ISSUES.md](DOCUMENT_SYNC_ISSUES.md) file.

## Project Roadmap

- [ ] Complete P2P network implementation with libp2p
- [ ] Enhanced LaTeX-specific features (syntax checking, autocompletion)
- [ ] Desktop client application
- [ ] Mobile clients
- [ ] Encrypted document support
- [ ] Collaborative LaTeX compilation
- [ ] Real-time preview and commenting

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Testing

We have several test scripts to verify different aspects of the system:

- `cargo run --bin comprehensive_test`: Tests all components together
- `cargo run --bin network_test`: Tests the network layer
- `cargo run --bin document_sync_test`: Specifically tests document synchronization
- `cargo run --bin advanced_sync_test`: Advanced synchronization testing

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- The [diamond-types](https://github.com/josephg/diamond-types) library for CRDT implementation
- [libp2p](https://github.com/libp2p/rust-libp2p) for P2P networking capabilities
- The Rust community for creating an amazing ecosystem
