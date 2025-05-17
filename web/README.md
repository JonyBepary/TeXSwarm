# P2P LaTeX Collaboration Tool - Web Frontend

This is a simple web-based frontend demo for the P2P LaTeX Collaboration Tool. It provides a user interface for creating, editing, and collaborating on LaTeX documents using the backend API.

## Features

- **Real-time collaborative editing** through WebSockets
- **Document management** (create, open, save)
- **User presence** indicators showing who's actively editing
- **Document sharing** for adding collaborators
- **LaTeX preview** to see rendered output

## Getting Started

### Prerequisites

- A running instance of the P2P LaTeX Collaboration Server
- Modern web browser (Chrome, Firefox, Safari, Edge)

### Setup

1. Ensure the server is running on the configured host and port
2. Open `index.html` in a web browser
3. Enter a username and click "Login"
4. Create a new document or open an existing one

### Configuration

The frontend can be configured by editing the constants at the top of `script.js`:

```javascript
const API_HOST = 'localhost';  // HTTP API host
const API_PORT = 8080;         // HTTP API port
const WS_HOST = 'localhost';   // WebSocket host
const WS_PORT = 8081;          // WebSocket port
```

## Usage

1. **Login**: Enter a username and click "Login"
2. **Create Document**: Click "New Document", enter a title, and optionally provide a Git repository URL
3. **Edit Document**: Use the editor to write LaTeX content
4. **Save Document**: Click "Save" to synchronize with Git
5. **Compile Document**: Click "Compile" to render the LaTeX preview
6. **Share Document**: Click "Share" to get a shareable link and manage collaborators

## Implementation Notes

This frontend demo uses:

- **Ace Editor** for the code editing experience
- **Bootstrap 5** for the UI components
- **WebSockets** for real-time communication with the server
- **Fetch API** for HTTP requests

## Limitations

This is a simplified demo and has some limitations:

- The LaTeX preview is a basic HTML representation, not a full LaTeX renderer
- User authentication is minimal (no passwords)
- Git repository integration requires server-side configuration
- Offline editing is not supported

## Next Steps

Future improvements could include:

- Full LaTeX rendering using MathJax or KaTeX
- Proper authentication and user management
- Conflict resolution visualizations
- File management within LaTeX projects
- Mobile-responsive design
