# TeXSwarm - Web Frontend

A modern web-based frontend for the TeXSwarm decentralized LaTeX collaboration platform. It provides an intuitive interface for creating, editing, and collaborating on LaTeX documents in real-time.

## Important Notice

**We have a new improved web app implementation!**

The original web app (index.html) had connectivity issues with showing "Offline" status and login problems. We've created a new web app with improved connectivity and a more robust implementation:

- **New Web App**: Use `new-index.html` for a better experience
- **New Server Script**: Use `serve_new_app.sh` to test the new web app
- **Documentation**: See `/docs/new_web_app.md` and `/docs/web_app_summary.md` for details

To use the new web app:
```bash
# Start the backend server
cd /jony/rust-pract/TeXSwarm
cargo run --bin p2p-latex-collab-server

# In another terminal, serve the web app
cd /jony/rust-pract/TeXSwarm/web
./serve_new_app.sh

# Access in browser
# http://localhost:8090/new-index.html
```

## Features

- **Real-time collaborative editing** through WebSockets
- **Document management** (create, open, save)
- **User presence** and cursor tracking for collaborative editing
- **LaTeX template selection** with various document types
- **Git repository integration** for version control
- **Live LaTeX preview** with MathJax rendering
- **Modern, responsive UI** with Bootstrap 5

### Configuration

The frontend can be configured by editing the constants at the top of `script.js`:

```javascript
const API_HOST = 'localhost';  // HTTP API host
const API_PORT = 8080;         // HTTP API port
const WS_HOST = 'localhost';   // WebSocket host
const WS_PORT = 8081;          // WebSocket port
```

## Usage

1. **Login**: Enter your name to start collaborating
2. **Create Document**:
   - Click the "+" button next to "My Documents"
   - Choose from Blank Document, Template, or Git Repository
   - Fill in the required information and click "Create Document"
3. **Edit Document**: Use the editor to write LaTeX content
4. **Save Document**: Click "Save" to save changes
5. **Compile**: Click "Compile" to render the LaTeX preview
6. **Share**: Click "Share" to get a shareable link and manage collaborators

## Connectivity Testing

If you're experiencing connection issues, follow these steps:

1. Make sure the backend server is running:
   ```bash
   cd /path/to/TeXSwarm
   cargo run --bin server
   ```

2. Check connectivity with the test tool:
   ```bash
   cd /path/to/TeXSwarm/web
   chmod +x serve_with_cors.sh
   ./serve_with_cors.sh
   ```

3. Open the connection test page in your browser:
   ```
   http://localhost:8000/connection_test.html
   ```

4. The test page will automatically check both HTTP and WebSocket connections.

### Common Issues and Solutions

1. **WebSocket connection fails**:
   - Make sure the server is running
   - Check if the correct ports are open (8080 for HTTP, 8081 for WebSocket)
   - Try using 127.0.0.1 instead of localhost
   - Check for firewall or network issues

2. **CORS (Cross-Origin Resource Sharing) errors**:
   - Use the provided `serve_with_cors.sh` script which enables CORS
   - Ensure both the web server and backend run on the same machine

3. **Server not found errors**:
   - Verify the host and port configurations in `script.js`
   - Check if the server is listening on the correct interface (0.0.0.0 vs 127.0.0.1)

4. **Authentication errors**:
   - Clear your browser cache and reload the page
   - Check for any session-related issues in the browser console

## Templates

The following LaTeX templates are available:

1. **Article** - For shorter academic papers, articles, or assignments
2. **Report** - For longer documents with chapters
3. **Book** - For full books with frontmatter, mainmatter, and backmatter
4. **Letter** - For formal correspondence
5. **Presentation** - For creating Beamer presentations

## Implementation Details

This frontend uses:

- **Ace Editor** for a powerful LaTeX editing experience
- **Bootstrap 5** for responsive UI components
- **WebSockets** for real-time collaboration
- **MathJax** for LaTeX rendering
- **Bootstrap Icons** for a beautiful icon set
- **Fetch API** for HTTP requests

## Recent Improvements

- Complete UI redesign with a modern, responsive interface
- Added modal login flow
- Enhanced document creation with templates
- Added MathJax for better formula rendering
- Improved user presence and collaboration features
- Added document templates for various LaTeX document types
- Fixed network implementation issues for better real-time collaboration

## Future Enhancements

Planned improvements include:

- Offline editing capability
- Full Git history visualization
- Enhanced LaTeX syntax highlighting
- Advanced collaboration features (comments, suggestions)
- Notifications for document changes
- Mobile app with the same collaboration features
- Authentication with user accounts and secure sharing
