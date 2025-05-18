# TeXSwarm New Web App Documentation

This document provides an overview of the new web application for TeXSwarm, which addresses connectivity issues and improves user experience.

## Overview

The new web application for TeXSwarm is a modern, responsive interface for collaborative LaTeX editing. It properly connects to the backend services, displays the online/offline status correctly, and includes enhanced error handling and reconnection mechanisms.

## Features

- **Robust Connection Management**:
  - Real-time connection status visibility
  - Automatic reconnection attempts
  - WebSocket heartbeat detection
  - Proper error handling

- **Improved User Experience**:
  - Clean, modern UI with Bootstrap 5
  - Better document management
  - Enhanced LaTeX editor with syntax highlighting
  - Document preview functionality

- **Authentication**:
  - User registration
  - Auto-login for returning users
  - Proper session management

## Getting Started

### Prerequisites

- Rust toolchain (for running the backend server)
- Web browser (Chrome, Firefox, Safari, or Edge)
- Python 3 (for serving the web app locally)

### Running the Application

1. Start the TeXSwarm backend server:
   ```bash
   cd /jony/rust-pract/TeXSwarm
   cargo run --bin p2p-latex-collab-server
   ```

2. Start the web app server:
   ```bash
   cd /jony/rust-pract/TeXSwarm/web
   ./serve_new_app.sh
   ```

3. Open a web browser and navigate to:
   ```
   http://localhost:8090/new-index.html
   ```

### Using the Application

1. **Login**: Enter a username to create an account or login with an existing one
2. **Create Document**: Click the "+" button in the Documents section to create a new document
3. **Edit Document**: Use the LaTeX editor to write your document
4. **Save Document**: Click the "Save" button to save your changes
5. **Compile Document**: Click the "Compile" button to compile your LaTeX document
6. **Share Document**: Click the "Share" button to get a link to share with collaborators

## File Structure

The new web app consists of the following files:

- `new-index.html` - Main HTML file with the application structure
- `new-styles.css` - Custom CSS styling for the application
- `new-app.js` - JavaScript code for application functionality
- `serve_new_app.sh` - Script to serve the application for testing

## Setup and Running

1. Make sure the TeXSwarm backend server is running:
   ```
   cargo run --bin server
   ```

2. Run the web server script:
   ```
   ./web/serve_new_app.sh
   ```

3. Open the application in your browser:
   - Go to `http://localhost:8000/new-index.html`

## Backend Connectivity

The web app connects to the TeXSwarm backend services using:

- HTTP API: `http://<host>:8080/api/`
- WebSocket: `ws://<host>:8081/ws`

## Connection Status Indicators

The web app provides clear visual indicators of connection status:

- **API Status**: Shows if the HTTP API is accessible
- **WS Status**: Shows if the WebSocket connection is established
- **Overall Status**: Combined status indicator in the sidebar

## User Flow

1. **Login**:
   - Enter a username and click "Login"
   - This creates a user via the API

2. **Creating Documents**:
   - Click "+" button in the Documents section
   - Enter a title and select a template
   - Click "Create"

3. **Editing Documents**:
   - Select a document from the list
   - Use the editor to make changes
   - Changes are automatically sent to the server

4. **Sharing Documents**:
   - Open a document
   - Click the "Share" button
   - Copy the generated URL

## Troubleshooting

- **Offline Status**:
  - Check if the server is running
  - Ensure your network connection is active
  - Try refreshing the page

- **Login Issues**:
  - Check the API status indicator
  - Ensure the server is running and accessible

- **WebSocket Disconnections**:
  - The app will automatically attempt to reconnect
  - Check your network connection
  - Restart the server if necessary

## Future Improvements

1. Add more LaTeX templates
2. Implement real-time collaborative editing visualization (cursors, etc.)
3. Add document version history
4. Enhance preview with proper LaTeX rendering
5. Implement user permissions and roles
