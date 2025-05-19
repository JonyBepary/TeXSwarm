/**
 * TeXSwarm - Collaborative LaTeX Editor
 * WebSocket Client Integration Example
 *
 * This file demonstrates how to integrate the WebSocket Protocol Client
 * into the main application. This is a partial file showing only the
 * WebSocket-related portions that would need to be updated.
 */

// Assuming the main application code is structured like new-app.js
// Here is how you would replace the WebSocket communication

// Configuration
const API_HOST = window.location.hostname || 'localhost';
const API_PORT = 8080;
const WS_HOST = window.location.hostname || 'localhost';
const WS_PORT = 8081;
const API_URL = `http://${API_HOST}:${API_PORT}`;
const WS_URL = `ws://${WS_HOST}:${WS_PORT}/ws`;

// State management
let currentUser = null;
let currentDocument = null;
let documents = [];
let collaborators = [];
let editor = null;
let wsClient = null; // Using WebSocketProtocolClient instead of raw websocket

/**
 * Connect to the WebSocket server
 */
function connectWebSocket() {
    // Create a WebSocket protocol client
    wsClient = new TeXSwarmWebSocketClient(WS_URL, {
        onOpen: handleWebSocketOpen,
        onMessage: handleWebSocketMessage,
        onClose: handleWebSocketClose,
        onError: handleWebSocketError,
        onStatusChange: updateConnectionStatus
    });

    // Connect
    wsClient.connect();
}

/**
 * Handle WebSocket open event
 */
function handleWebSocketOpen(event) {
    console.log('WebSocket connected');
    updateConnectionStatus('connected');
    document.getElementById('ws-status').textContent = 'Connected';
    document.getElementById('ws-status').classList.add('text-success');
    document.getElementById('ws-status').classList.remove('text-danger');

    // Send authentication if we have a user
    if (currentUser) {
        authenticateWebSocket();
    }
}

/**
 * Handle WebSocket message event
 * This stays the same since we receive parsed messages in the same format
 */
function handleWebSocketMessage(message, event) {
    // Message processing as before
    // Now we get the parsed message object directly
    switch (message.type) {
        case 'Heartbeat':
            handleHeartbeat(message.payload);
            break;
        case 'DocumentUpdate':
            handleDocumentUpdate(message.payload);
            break;
        case 'DocumentList':
            handleDocumentList(message.payload);
            break;
        case 'PresenceUpdate':
            handlePresenceUpdate(message.payload);
            break;
        case 'Error':
            handleError(message.payload);
            break;
        default:
            console.log('Unknown message type:', message.type);
    }
}

/**
 * Handle WebSocket close event
 */
function handleWebSocketClose(event) {
    console.log('WebSocket disconnected:', event.code, event.reason);
    updateConnectionStatus('offline');
    document.getElementById('ws-status').textContent = 'Disconnected';
    document.getElementById('ws-status').classList.remove('text-success');
    document.getElementById('ws-status').classList.add('text-danger');
}

/**
 * Handle WebSocket error event
 */
function handleWebSocketError(error) {
    console.error('WebSocket error:', error);
    document.getElementById('ws-status').textContent = 'Error';
    document.getElementById('ws-status').classList.add('text-danger');
}

/**
 * Send authentication via WebSocket
 */
function authenticateWebSocket() {
    if (!wsClient || !currentUser) return;

    // Use the protocol client's authenticate method
    wsClient.authenticate(currentUser.id, currentUser.token);

    // Request the list of documents after authentication
    wsClient.listDocuments();
}

/**
 * Send a document operation
 */
function sendDocumentOperation() {
    if (!wsClient || !currentDocument) return;

    // Get content from editor
    const content = editor.getValue();

    // Use the protocol client's method
    wsClient.replaceDocumentContent(currentDocument.id, content);
}

/**
 * Create a new document
 */
function createDocument(title, callback) {
    if (!wsClient) return;

    // Use the protocol client's method
    wsClient.createDocument(title);
}

/**
 * Open a document
 */
function openDocument(documentId) {
    if (!wsClient) return;

    // Use the protocol client's method
    wsClient.openDocument(documentId);
}

/**
 * Check if WebSocket is connected
 */
function isWebSocketConnected() {
    return wsClient && wsClient.isConnected();
}

/**
 * Update connection status in UI
 */
function updateConnectionStatus(status) {
    const statusIndicator = document.getElementById('status-indicator');
    const statusText = document.getElementById('status-text');

    statusIndicator.className = 'status-indicator';

    switch (status) {
        case 'connecting':
            statusIndicator.classList.add('status-connecting');
            statusText.textContent = 'Connecting...';
            break;
        case 'connected':
            statusIndicator.classList.add('status-connected');
            statusText.textContent = 'Connected';
            break;
        case 'disconnected':
        case 'offline':
            statusIndicator.classList.add('status-disconnected');
            statusText.textContent = 'Disconnected';
            break;
        case 'error':
            statusIndicator.classList.add('status-error');
            statusText.textContent = 'Error';
            break;
        default:
            statusIndicator.classList.add('status-disconnected');
            statusText.textContent = status;
    }
}
