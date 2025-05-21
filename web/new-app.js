/**
 * TeXSwarm - Collaborative LaTeX Editor
 * This is the main JavaScript file for the application.
 */

// Configuration
const API_HOST = window.location.hostname || 'localhost';
let API_PORT = 8090; // Default, will be overridden if config.json is available
const WS_HOST = window.location.hostname || 'localhost';
let WS_PORT = 8091; // Default, will be overridden if config.json is available
let API_URL = `http://${API_HOST}:${API_PORT}`;
let WS_URL = `ws://${WS_HOST}:${WS_PORT}/ws`;

// Function to load configuration from config.json
async function loadConfig() {
    try {
        console.log(`Attempting to load config from ../config.json`);
        const response = await fetch('../config.json');
        if (response.ok) {
            const config = await response.json();
            if (config && config.server) {
                API_PORT = config.server.api_port || API_PORT;
                WS_PORT = config.server.ws_port || WS_PORT;

                // Update URLs with new port values
                API_URL = `http://${API_HOST}:${API_PORT}`;
                WS_URL = `ws://${WS_HOST}:${WS_PORT}/ws`;

                console.log(`Configuration loaded: API Port=${API_PORT}, WS Port=${WS_PORT}`);
                console.log(`Updated API_URL=${API_URL}, WS_URL=${WS_URL}`);
            }
        } else {
            console.warn(`Failed to load config.json (status: ${response.status}), using default ports`);
        }
    } catch (error) {
        console.error('Error loading config:', error);
    }

    // Update any elements that display port information
    updatePortDisplay();
}

// Function to update any elements that display port information
function updatePortDisplay() {
    // If we have elements showing connection info, update them here
    const connectionInfoEl = document.getElementById('connection-info');
    if (connectionInfoEl) {
        connectionInfoEl.textContent = `Connected to: API on port ${API_PORT}, WebSocket on port ${WS_PORT}`;
    }

    // Update the global URL variables to match the current ports
    API_URL = `http://${API_HOST}:${API_PORT}`;
    WS_URL = `ws://${WS_HOST}:${WS_PORT}/ws`;

    console.log(`Updated URLs: API_URL=${API_URL}, WS_URL=${WS_URL}`);
}

// State management
let currentUser = null;
let currentDocument = null;
let documents = [];
let collaborators = [];
let editor = null;
let websocket = null;
let connectionTimeout = null;
let heartbeatTimeout = null;
let heartbeatInterval = null;
let lastHeartbeat = null;

/**
 * Initialize the application when the DOM is loaded
 */
document.addEventListener('DOMContentLoaded', async () => {
    // Load configuration from config.json
    await loadConfig();

    // Initialize Ace editor
    initializeEditor();

    // Initialize Bootstrap components
    initializeModals();

    // Setup event listeners
    setupEventListeners();

    // Initialize connection status
    updateConnectionStatus('connecting');

    // Check API connection
    checkApiConnection();

    // Try to reconnect websocket when page becomes visible
    document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'visible' && !isWebSocketConnected()) {
            connectWebSocket();
        }
    });

    // Handle page refresh or closing
    window.addEventListener('beforeunload', () => {
        if (websocket && websocket.readyState === WebSocket.OPEN) {
            websocket.close();
        }
    });

    // Parse URL parameters for document sharing
    handleUrlParameters();
});

/**
 * Initialize the Ace editor
 */
function initializeEditor() {
    editor = ace.edit('editor');
    editor.setTheme('ace/theme/monokai');
    editor.session.setMode('ace/mode/latex');
    editor.setShowPrintMargin(false);
    editor.setOptions({
        fontSize: '14pt',
        enableBasicAutocompletion: true,
        enableSnippets: true,
        enableLiveAutocompletion: true
    });

    // When editor content changes, debounce and send updates
    editor.getSession().on('change', debounce(handleEditorChange, 500));
}

/**
 * Initialize Bootstrap modals
 */
function initializeModals() {
    // Currently initialized via Bootstrap's auto-initialization
}

/**
 * Setup event listeners for buttons and forms
 */
function setupEventListeners() {
    // Login form submission
    document.getElementById('login-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const username = document.getElementById('username-input').value.trim();
        if (username) {
            await login(username);
        }
    });

    // Logout button
    document.getElementById('logout-btn').addEventListener('click', logout);

    // Refresh connection button
    document.getElementById('refresh-connection').addEventListener('click', (e) => {
        e.preventDefault();
        console.log('Manually refreshing connection');
        checkApiConnection();
    });

    // New document button
    document.getElementById('new-doc-btn').addEventListener('click', () => {
        if (!currentUser) {
            showToast('Please login first');
            return;
        }
        // Reset modal fields
        document.getElementById('doc-title').value = '';
        document.getElementById('template-select').value = 'article';

        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('new-doc-modal'));
        modal.show();
    });

    // Create document button
    document.getElementById('create-doc-btn').addEventListener('click', () => {
        const title = document.getElementById('doc-title').value.trim();
        const template = document.getElementById('template-select').value;

        if (!title) {
            showToast('Please enter a document title');
            return;
        }

        createDocument(title, template);

        // Hide modal
        const modal = bootstrap.Modal.getInstance(document.getElementById('new-doc-modal'));
        modal.hide();
    });

    // Save button
    document.getElementById('save-btn').addEventListener('click', () => {
        if (currentDocument) {
            saveDocument();
        }
    });

    // Compile button
    document.getElementById('compile-btn').addEventListener('click', () => {
        if (currentDocument) {
            compileDocument();
        }
    });

    // Share button
    document.getElementById('share-btn').addEventListener('click', () => {
        if (!currentDocument) {
            showToast('Please select a document first');
            return;
        }

        // Set share URL
        const shareUrl = `${window.location.origin}${window.location.pathname}?doc=${currentDocument.id}`;
        document.getElementById('share-url').value = shareUrl;

        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('share-modal'));
        modal.show();
    });

    // Debug reconnect button
    document.getElementById('debug-reconnect').addEventListener('click', () => {
        console.log("Debug: Manual reconnection requested");
        showToast("Reconnecting to server...");

        // Close the existing connection if open
        if (websocket && websocket.readyState === WebSocket.OPEN) {
            websocket.close();
        }

        // Force reload configuration and reconnect
        loadConfig().then(() => {
            checkApiConnection();
        });
    });

    // Debug create document button
    document.getElementById('debug-create-doc').addEventListener('click', () => {
        console.log("Debug: Creating test document");
        showToast("Creating a test document...");

        const testDocTitle = `Test Doc ${new Date().toISOString().slice(0, 16)}`;
        createDocument(testDocTitle, "basic");
    });

    // Debug fix document branches button
    document.getElementById('debug-fix-branches').addEventListener('click', () => {
        console.log("Debug: Attempting to fix document branches");
        showToast("Attempting to fix document branches...");

        if (currentDocument && currentDocument.id) {
            // Try to create the document branch if it doesn't exist
            fixDocumentBranch(currentDocument.id);
        } else {
            showToast("No document selected. Please open a document first.", "warning");
        }
    });

    // Debug create branch button
    document.getElementById('debug-create-branch').addEventListener('click', () => {
        console.log("Debug: Creating document branch explicitly");
        showToast("Creating document branch explicitly...");

        if (currentDocument && currentDocument.id) {
            // Try to create the document branch explicitly
            if (typeof createDocumentBranch === 'function') {
                createDocumentBranch(currentDocument.id);
            } else {
                // Fallback if the function isn't available
                const message = {
                    type: 'CreateDocumentBranch',
                    payload: {
                        document_id: currentDocument.id
                    }
                };

                if (websocket && websocket.readyState === WebSocket.OPEN) {
                    try {
                        websocket.send(JSON.stringify(message));
                        console.log("Sent CreateDocumentBranch message");
                    } catch (e) {
                        console.error("Failed to send message:", e);
                        showToast("Failed to create branch", "error");
                    }
                } else {
                    showToast("WebSocket not connected", "error");
                }
            }
        } else {
            showToast("No document selected. Please open a document first.", "warning");
        }
    });

    // Copy URL button
    document.getElementById('copy-url-btn').addEventListener('click', () => {
        const shareUrl = document.getElementById('share-url');
        shareUrl.select();
        navigator.clipboard.writeText(shareUrl.value);

        const btn = document.getElementById('copy-url-btn');
        const originalHtml = btn.innerHTML;
        btn.innerHTML = '<i class="bi bi-check-lg"></i>';

        setTimeout(() => {
            btn.innerHTML = originalHtml;
        }, 1500);
    });

    // Refresh preview button
    document.getElementById('refresh-preview-btn').addEventListener('click', () => {
        if (currentDocument) {
            compileDocument();
        }
    });
}

/**
 * Handle URL parameters for shared documents
 */
function handleUrlParameters() {
    const urlParams = new URLSearchParams(window.location.search);
    const docId = urlParams.get('doc');

    if (docId) {
        // If we have a document ID in the URL, we'll try to open it after login
        localStorage.setItem('pendingDocumentId', docId);
    }
}

/**
 * Check the API connection by pinging the server
 */
async function checkApiConnection() {
    try {
        // Make sure we're using the most up-to-date URLs
        API_URL = `http://${API_HOST}:${API_PORT}`;
        WS_URL = `ws://${WS_HOST}:${WS_PORT}/ws`;

        console.log(`Checking API connection to ${API_URL}/api/ping`);
        console.log(`Current WebSocket URL: ${WS_URL}`);

        const response = await fetch(`${API_URL}/api/ping`);

        if (response.ok) {
            const data = await response.json();
            console.log(`API connection successful:`, data);

            document.getElementById('api-status').textContent = 'Connected';
            document.getElementById('api-status').classList.add('text-success');

            // Once API is confirmed, try to connect to WebSocket
            connectWebSocket();

            // Try to auto-login if we have stored credentials
            const savedUserId = localStorage.getItem('userId');
            const savedUsername = localStorage.getItem('username');

            if (savedUserId && savedUsername) {
                autoLogin(savedUserId, savedUsername);
            }
        } else {
            console.error(`API responded with status: ${response.status}`);
            document.getElementById('api-status').textContent = 'Error';
            document.getElementById('api-status').classList.add('text-danger');
            updateConnectionStatus('offline');
        }
    } catch (error) {
        console.error('API connection error:', error);
        document.getElementById('api-status').textContent = 'Offline';
        document.getElementById('api-status').classList.add('text-danger');
        updateConnectionStatus('offline');

        // Retry connection after 5 seconds
        setTimeout(checkApiConnection, 5000);
    }
}

/**
 * Connect to the WebSocket server
 */
function connectWebSocket() {
    // Clear any existing connection timeout
    if (connectionTimeout) {
        clearTimeout(connectionTimeout);
    }

    // Update connection status
    updateConnectionStatus('connecting');
    document.getElementById('ws-status').textContent = 'Connecting...';

    try {
        // Make sure we're using the current URL with the latest port from config
        const currentWsUrl = `ws://${WS_HOST}:${WS_PORT}/ws`;
        console.log(`Connecting to WebSocket at ${currentWsUrl}`);

        websocket = new WebSocket(currentWsUrl);

        // Setup WebSocket event handlers
        websocket.onopen = handleWebSocketOpen;
        websocket.onmessage = handleWebSocketMessage;
        websocket.onclose = handleWebSocketClose;
        websocket.onerror = handleWebSocketError;

        // Set a connection timeout
        connectionTimeout = setTimeout(() => {
            if (websocket.readyState !== WebSocket.OPEN) {
                websocket.close();
                updateConnectionStatus('offline');
                document.getElementById('ws-status').textContent = 'Timeout';
                document.getElementById('ws-status').classList.add('text-danger');

                // Retry connection after 5 seconds
                setTimeout(connectWebSocket, 5000);
            }
        }, 10000);
    } catch (error) {
        console.error('WebSocket connection error:', error);
        updateConnectionStatus('offline');
        document.getElementById('ws-status').textContent = 'Error';
        document.getElementById('ws-status').classList.add('text-danger');

        // Retry connection after 5 seconds
        setTimeout(connectWebSocket, 5000);
    }
}

/**
 * Handle WebSocket open event
 */
function handleWebSocketOpen() {
    console.log('WebSocket connected');
    updateConnectionStatus('online');
    document.getElementById('ws-status').textContent = 'Connected';
    document.getElementById('ws-status').classList.remove('text-danger');
    document.getElementById('ws-status').classList.add('text-success');

    // Clear connection timeout
    if (connectionTimeout) {
        clearTimeout(connectionTimeout);
        connectionTimeout = null;
    }

    // Authenticate if we have a user
    if (currentUser) {
        sendAuthentication();
    }

    // Setup heartbeat checking
    setupHeartbeatCheck();
}

/**
 * Handle WebSocket message
 */
function handleWebSocketMessage(event) {
    try {
        const message = JSON.parse(event.data);

        switch (message.type) {
            case 'Ping':
                handlePing();
                break;
            case 'DocumentList':
                handleDocumentList(message.payload);
                break;
            case 'DocumentContent':
                handleDocumentContent(message.payload);
                break;
            case 'DocumentUpdate':
                handleDocumentUpdate(message.payload);
                break;
            case 'CollaboratorList':
                handleCollaboratorList(message.payload);
                break;
            case 'UserAuthenticated':
                handleUserAuthenticated(message.payload);
                break;
            case 'CompileResult':
                handleCompileResult(message.payload);
                break;
            case 'BranchCreated':
                handleBranchCreated(message.payload);
                break;
            case 'Error':
                handleErrorMessage(message.payload);
                break;
            default:
                console.log('Unhandled message type:', message.type);
        }
    } catch (error) {
        console.error('Error handling WebSocket message:', error, event.data);
    }
}

/**
 * Handle branch created message
 */
function handleBranchCreated(payload) {
    if (!payload || !payload.document_id) {
        console.warn('Invalid BranchCreated payload:', payload);
        return;
    }

    console.log(`Document branch created for ${payload.document_id}`);

    // If we have the status update function available
    if (typeof updateDocumentBranchStatus === 'function') {
        updateDocumentBranchStatus(payload.document_id, 'fixed');
    }

    // If this is for the current document or a pending document, try to open it
    if (currentDocument && currentDocument.id === payload.document_id) {
        console.log('Branch created for current document, refreshing content');
        requestDocumentContent(payload.document_id);
    } else {
        // Check if there's a pending document to open
        const pendingDocId = localStorage.getItem('pendingDocumentId');
        if (pendingDocId === payload.document_id) {
            console.log('Branch created for pending document, opening it now');
            openDocument(pendingDocId);
            localStorage.removeItem('pendingDocumentId');
        }
    }

    showToast('Document branch created successfully', 'success');
}

/**
 * Handle error message
 */
function handleErrorMessage(payload) {
    if (!payload || !payload.message) {
        console.warn('Invalid Error payload:', payload);
        return;
    }

    const errorMessage = payload.message;
    console.error('Server error:', errorMessage);

    // Check for specific error types
    if (errorMessage.includes('Document branch not found')) {
        // Extract document ID if possible
        const match = errorMessage.match(/Document branch not found: ([0-9a-f-]{36})/);
        const documentId = match ? match[1] : null;

        if (documentId) {
            console.log(`Document branch error for ${documentId}`);

            // If we have the document branch fix function available
            if (typeof fixDocumentBranch === 'function') {
                console.log('Attempting automatic fix via fixDocumentBranch');
                fixDocumentBranch(documentId);

                // Don't show the error toast since we're handling it
                return;
            }
        }
    }

    // For any other errors, show a toast
    showToast(`Error: ${errorMessage}`, 'error');
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

    // Clear heartbeat checking
    clearHeartbeatCheck();

    // Attempt to reconnect after a delay
    setTimeout(connectWebSocket, 5000);
}

/**
 * Handle WebSocket error event
 */
function handleWebSocketError(error) {
    console.error('WebSocket error:', error);
    updateConnectionStatus('offline');
    document.getElementById('ws-status').textContent = 'Error';
    document.getElementById('ws-status').classList.remove('text-success');
    document.getElementById('ws-status').classList.add('text-danger');
}

/**
 * Setup heartbeat checking to detect stale connections
 */
function setupHeartbeatCheck() {
    // Clear any existing heartbeat checks
    clearHeartbeatCheck();

    // Set a timeout to verify we receive heartbeats
    heartbeatTimeout = setTimeout(() => {
        // If we haven't received a heartbeat in 35 seconds, connection is stale
        const now = new Date().getTime();
        if (!lastHeartbeat || (now - lastHeartbeat) > 35000) {
            console.log('No heartbeat received, reconnecting...');
            if (websocket) {
                websocket.close();
            }
        }
    }, 35000);
}

/**
 * Clear heartbeat checking
 */
function clearHeartbeatCheck() {
    if (heartbeatTimeout) {
        clearTimeout(heartbeatTimeout);
        heartbeatTimeout = null;
    }

    if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
        heartbeatInterval = null;
    }
}

/**
 * Handle heartbeat message from server
 */
function handleHeartbeat(payload) {
    lastHeartbeat = new Date().getTime();
    updateConnectionStatus('online');
}

/**
 * Fix a document branch by calling the Document Persistence API
 * This will create the branch if it doesn't exist
 */
async function fixDocumentBranch(documentId) {
    try {
        console.log(`Fixing document branch for ${documentId}`);

        // Use the Document Persistence API endpoint to check/create the document
        const response = await fetch(`${API_URL}/api/documents/${documentId}/check`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json',
            }
        });

        if (response.ok) {
            const result = await response.json();
            console.log('Document branch fix result:', result);
            showToast(result.message);

            // If the document was created or exists, try to open it again
            if (result.success) {
                // Wait a moment to ensure the server has time to process
                setTimeout(() => {
                    openDocument(documentId);
                }, 1000);
            }
        } else {
            const errorText = await response.text();
            console.error('Error fixing document branch:', errorText);
            showToast(`Error fixing document branch: ${response.status}`, 'error');
        }
    } catch (error) {
        console.error('Exception fixing document branch:', error);
        showToast(`Exception fixing document branch: ${error.message}`, 'error');
    }
}

/**
 * Send authentication message to WebSocket
 */
function sendAuthentication() {
    if (websocket && websocket.readyState === WebSocket.OPEN && currentUser) {
        const authMessage = {
            type: 'Authentication',
            payload: {
                user_id: currentUser.id,
                token: null // We're not using tokens yet
            }
        };

        websocket.send(JSON.stringify(authMessage));
    }
}

/**
 * Handle document update from server
 */
function handleDocumentUpdate(payload) {
    if (currentDocument && payload.document_id === currentDocument.id) {
        // Update editor content without triggering change event
        const currentPosition = editor.getCursorPosition();
        editor.session.doc.setValue(payload.content);
        editor.moveCursorToPosition(currentPosition);

        // Update document version
        currentDocument.version = payload.version;
    }
}

/**
 * Handle document list from server
 */
function handleDocumentList(payload) {
    documents = payload.documents;
    updateDocumentList();
}

/**
 * Handle presence update from server
 */
function handlePresenceUpdate(payload) {
    if (currentDocument && payload.document_id === currentDocument.id) {
        collaborators = payload.presence;
        updateCollaboratorList();
    }
}

/**
 * Handle error message from server
 */
function handleError(payload) {
    // Check if this is actually a success message disguised as an error
    // This happens with our authentication success message
    if (payload.code === "auth_success") {
        console.log('Authentication success:', payload.message);
        // Request document list after successful authentication
        requestDocumentList();
        return;
    }

    showToast(`Error: ${payload.message}`, 'error');
}

/**
 * Update connection status UI
 */
function updateConnectionStatus(status) {
    const indicator = document.getElementById('status-indicator');
    const statusText = document.getElementById('status-text');

    indicator.className = 'status-indicator';

    switch (status) {
        case 'online':
            indicator.classList.add('online');
            statusText.textContent = 'Online';
            break;
        case 'offline':
            indicator.classList.add('offline');
            statusText.textContent = 'Offline';
            break;
        case 'connecting':
            statusText.textContent = 'Connecting...';
            break;
    }
}

/**
 * Login user
 */
async function login(username) {
    try {
        const response = await fetch(`${API_URL}/api/users`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ name: username })
        });

        if (!response.ok) {
            throw new Error('Login failed');
        }

        const data = await response.json();

        // Save user information
        currentUser = {
            id: data.id,
            name: username
        };

        // Store in localStorage for persistence
        localStorage.setItem('userId', data.id);
        localStorage.setItem('username', username);

        // Update UI
        document.getElementById('login-section').classList.add('d-none');
        document.getElementById('user-section').classList.remove('d-none');
        document.getElementById('user-name').textContent = username;
        document.getElementById('user-id').textContent = `ID: ${data.id.substring(0, 8)}...`;
        document.getElementById('user-avatar').textContent = getInitials(username);

        // Enable editing controls
        toggleEditingControls(true);

        // Authenticate WebSocket if connected
        if (isWebSocketConnected()) {
            sendAuthentication();
        }

        // Request document list
        requestDocumentList();

        // Check if we have a pending document to open
        const pendingDocId = localStorage.getItem('pendingDocumentId');
        if (pendingDocId) {
            openDocument(pendingDocId);
            localStorage.removeItem('pendingDocumentId');
        }

        showToast(`Welcome, ${username}!`);
    } catch (error) {
        console.error('Login error:', error);
        showToast('Login failed. Please try again.', 'error');
    }
}

/**
 * Auto-login with saved credentials
 */
async function autoLogin(userId, username) {
    // Don't attempt auto-login if connection is offline
    if (!isWebSocketConnected()) {
        return;
    }

    currentUser = {
        id: userId,
        name: username
    };

    // Update UI
    document.getElementById('login-section').classList.add('d-none');
    document.getElementById('user-section').classList.remove('d-none');
    document.getElementById('user-name').textContent = username;
    document.getElementById('user-id').textContent = `ID: ${userId.substring(0, 8)}...`;
    document.getElementById('user-avatar').textContent = getInitials(username);

    // Enable editing controls
    toggleEditingControls(true);

    // Authenticate WebSocket
    sendAuthentication();

    // Request document list
    requestDocumentList();

    // Check if we have a pending document to open
    const pendingDocId = localStorage.getItem('pendingDocumentId');
    if (pendingDocId) {
        openDocument(pendingDocId);
        localStorage.removeItem('pendingDocumentId');
    }

    showToast(`Welcome back, ${username}!`);
}

/**
 * Logout user
 */
function logout() {
    // Clear user information
    currentUser = null;
    localStorage.removeItem('userId');
    localStorage.removeItem('username');

    // Update UI
    document.getElementById('login-section').classList.remove('d-none');
    document.getElementById('user-section').classList.add('d-none');
    document.getElementById('user-name').textContent = 'Not logged in';
    document.getElementById('user-id').textContent = '';
    document.getElementById('user-avatar').textContent = '';

    // Disable editing controls
    toggleEditingControls(false);

    // Close current document
    closeDocument();

    // Clear document list
    documents = [];
    updateDocumentList();

    // Check connection status and reconnect if needed
    checkApiConnection();

    showToast('You have been logged out');
}

/**
 * Toggle editing controls
 */
function toggleEditingControls(enabled) {
    document.getElementById('save-btn').disabled = !enabled;
    document.getElementById('compile-btn').disabled = !enabled;
    document.getElementById('share-btn').disabled = !enabled;
    editor.setReadOnly(!enabled);
}

/**
 * Request document list from server
 */
function requestDocumentList() {
    if (isWebSocketConnected() && currentUser) {
        const message = {
            type: 'ListDocuments'
        };

        websocket.send(JSON.stringify(message));
    }
}

/**
 * Update document list in UI
 */
function updateDocumentList() {
    const list = document.getElementById('document-list');

    // Clear current list
    list.innerHTML = '';

    if (documents.length === 0) {
        const emptyItem = document.createElement('li');
        emptyItem.className = 'empty-list';
        emptyItem.textContent = 'No documents yet';
        list.appendChild(emptyItem);
        return;
    }

    // Add each document to the list
    documents.forEach(doc => {
        const item = document.createElement('li');
        item.textContent = doc.title;
        item.setAttribute('data-document-id', doc.id);

        if (currentDocument && doc.id === currentDocument.id) {
            item.classList.add('active');
        }

        item.addEventListener('click', () => {
            openDocument(doc.id);
        });

        list.appendChild(item);
    });
}

/**
 * Update collaborator list in UI
 */
function updateCollaboratorList() {
    const list = document.getElementById('collaborator-list');

    // Clear current list
    list.innerHTML = '';

    if (!currentDocument || collaborators.length === 0) {
        const emptyItem = document.createElement('li');
        emptyItem.className = 'empty-list';
        emptyItem.textContent = 'No active collaborators';
        list.appendChild(emptyItem);
        return;
    }

    // Add each collaborator to the list
    collaborators.forEach(collab => {
        // Skip current user
        if (currentUser && collab.user_id === currentUser.id) {
            return;
        }

        const item = document.createElement('li');

        const avatar = document.createElement('div');
        avatar.className = 'user-avatar';
        avatar.style.backgroundColor = getRandomColor(collab.user_id);
        avatar.textContent = getInitials(collab.name || 'User');

        const name = document.createElement('span');
        name.className = 'ms-2';
        name.textContent = collab.name || collab.user_id.substring(0, 8);

        item.appendChild(avatar);
        item.appendChild(name);
        list.appendChild(item);
    });
}

/**
 * Create a new document
 */
async function createDocument(title, templateType) {
    if (!currentUser) {
        showToast('Please login first', 'error');
        return;
    }

    try {
        console.log(`Creating document "${title}" with template "${templateType}"`);
        showToast(`Creating document "${title}"...`);

        // Generate content based on template
        const content = getLatexTemplate(templateType, title, currentUser.name);

        // Send request to create document
        const response = await fetch(`${API_URL}/api/documents`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                title: title,
                owner_id: currentUser.id,
                content: content
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Failed to create document: ${response.status} - ${errorText}`);
            throw new Error(`Failed to create document: ${response.status}`);
        }

        const data = await response.json();
        console.log('Document created successfully:', data);

        // Request updated document list
        requestDocumentList();

        // Fix the document branch before opening it
        await fixDocumentBranch(data.document_id);

        // Open the new document
        setTimeout(() => {
            openDocument(data.document_id);
        }, 1000);

        showToast(`Document "${title}" created`);
    } catch (error) {
        console.error('Document creation error:', error);
        showToast('Failed to create document', 'error');
    }
}

/**
 * Open a document by ID
 */
function openDocument(documentId) {
    if (!isWebSocketConnected()) {
        showToast('WebSocket not connected', 'error');
        return;
    }

    if (!currentUser) {
        showToast('Please login first', 'error');
        return;
    }

    console.log(`Attempting to open document: ${documentId}`);

    // Update document branch status if we have the function available
    if (typeof updateDocumentBranchStatus === 'function') {
        updateDocumentBranchStatus(documentId, 'unknown');
    }

    // Clear editor
    if (editor) {
        editor.setValue('');
    }

    // Show loading state
    document.getElementById('editor-container').classList.add('loading');

    // Send open document message
    const message = {
        type: 'OpenDocument',
        payload: {
            document_id: documentId
        }
    };

    try {
        websocket.send(JSON.stringify(message));

        // Track opening time for timeout handling
        const openStartTime = Date.now();
        const openTimeout = 5000; // 5 seconds

        // Set up a timeout to check if opening succeeded
        const openDocTimeout = setTimeout(() => {
            console.log(`Document open timeout reached for ${documentId}`);

            // Check if document was successfully opened
            const isOpen = currentDocument && currentDocument.id === documentId;
            if (!isOpen) {
                console.log(`Document didn't open within timeout period`);

                // Try to fix document branch if the function is available
                if (typeof fixDocumentBranch === 'function') {
                    console.log(`Attempting to fix document branch for ${documentId}`);
                    fixDocumentBranch(documentId);

                    // Clear loading state
                    document.getElementById('editor-container').classList.remove('loading');

                    // Show message to user
                    showToast('Attempting to fix document branch...', 'info');
                }
            }
        }, openTimeout);

        // Store timeout so it can be cleared if open succeeds
        window._openDocTimeoutId = openDocTimeout;

        console.log('Sent OpenDocument message');

    } catch (error) {
        console.error('Failed to send OpenDocument message:', error);
        document.getElementById('editor-container').classList.remove('loading');
        showToast('Failed to open document', 'error');
    }
}

/**
 * Close current document
 */
function closeDocument() {
    currentDocument = null;
    collaborators = [];
    document.getElementById('document-title').textContent = 'No document selected';
    editor.session.setValue('% Select or create a document');
    updateDocumentList();
    updateCollaboratorList();
}

/**
 * Save current document
 */
async function saveDocument() {
    if (!currentUser || !currentDocument) {
        showToast('No active document to save', 'error');
        return;
    }

    try {
        const content = editor.getValue();

        // Send request to save document
        const response = await fetch(`${API_URL}/api/documents/${currentDocument.id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'x-user-id': currentUser.id
            },
            body: JSON.stringify({
                content: content
            })
        });

        if (!response.ok) {
            throw new Error('Failed to save document');
        }

        showToast('Document saved successfully');
    } catch (error) {
        console.error('Document saving error:', error);
        showToast('Failed to save document', 'error');
    }
}

/**
 * Compile current document
 */
function compileDocument() {
    if (!currentDocument) {
        showToast('No active document to compile', 'error');
        return;
    }

    // Get the current document content
    const content = editor.getValue();

    // For now, we'll just render a simple preview
    // In a real app, this would call a LaTeX compilation service

    const previewContent = document.querySelector('.preview-content');
    previewContent.innerHTML = '<div class="text-center p-4"><div class="spinner-border" role="status"></div><p class="mt-2">Compiling...</p></div>';

    // Simulate compilation delay
    setTimeout(() => {
        previewContent.innerHTML = `
            <h1>${currentDocument.title}</h1>
            <div class="latex-preview">
                <pre>${content}</pre>
            </div>
        `;

        // Initialize MathJax to render any LaTeX formulas
        if (window.MathJax) {
            window.MathJax.typeset();
        }
    }, 1500);
}

/**
 * Handle editor content changes
 */
function handleEditorChange(event) {
    if (!currentUser || !currentDocument || !isWebSocketConnected()) {
        return;
    }

    // Send document update via WebSocket
    const content = editor.getValue();
    const message = {
        type: 'DocumentOperation',
        payload: {
            operation: {
                document_id: currentDocument.id,
                user_id: currentUser.id,
                content: content,
                type: 'Update'
            }
        }
    };

    websocket.send(JSON.stringify(message));
}

/**
 * Check if WebSocket is connected
 */
function isWebSocketConnected() {
    return websocket && websocket.readyState === WebSocket.OPEN;
}

/**
 * Get LaTeX template based on template type
 */
function getLatexTemplate(templateType, title, author) {
    const templates = {
        article: `\\documentclass{article}
\\usepackage[utf8]{inputenc}
\\usepackage{amsmath}
\\usepackage{amssymb}

\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\maketitle

\\section{Introduction}
Your introduction here.

\\section{Background}
Background information here.

\\section{Methods}
Your methods here.

\\section{Results}
Your results here.

\\section{Discussion}
Discussion here.

\\section{Conclusion}
Conclusion here.

\\end{document}`,

        report: `\\documentclass{report}
\\usepackage[utf8]{inputenc}
\\usepackage{amsmath}
\\usepackage{amssymb}

\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\maketitle
\\tableofcontents

\\chapter{Introduction}
Your introduction here.

\\chapter{Background}
Background information here.

\\chapter{Methods}
Your methods here.

\\chapter{Results}
Your results here.

\\chapter{Discussion}
Discussion here.

\\chapter{Conclusion}
Conclusion here.

\\end{document}`,

        book: `\\documentclass{book}
\\usepackage[utf8]{inputenc}
\\usepackage{amsmath}
\\usepackage{amssymb}

\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\maketitle
\\tableofcontents

\\chapter{Introduction}
Your introduction here.

\\chapter{Main Content}
Your content here.

\\chapter{Conclusion}
Conclusion here.

\\end{document}`,

        letter: `\\documentclass{letter}
\\usepackage[utf8]{inputenc}
\\address{Sender's Address \\\\ City, State ZIP}
\\signature{${author}}

\\begin{document}

\\begin{letter}{Recipient's Name \\\\ Recipient's Address \\\\ City, State ZIP}

\\opening{Dear Sir or Madam,}

This is the content of the letter.

\\closing{Sincerely,}

\\end{letter}

\\end{document}`,

        blank: `% ${title}
% Author: ${author}
% Date: \\today

\\documentclass{article}
\\usepackage[utf8]{inputenc}

\\begin{document}

% Start your document here

\\end{document}`
    };

    return templates[templateType] || templates.article;
}

/**
 * Get user initials for avatar
 */
function getInitials(name) {
    if (!name) return '?';

    const parts = name.trim().split(' ');
    if (parts.length === 1) {
        return parts[0].charAt(0).toUpperCase();
    } else {
        return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase();
    }
}

/**
 * Get a consistent random color based on a string
 */
function getRandomColor(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }

    const hue = Math.abs(hash % 360);
    return `hsl(${hue}, 70%, 45%)`;
}

/**
 * Show a toast notification
 */
function showToast(message, type = 'info') {
    // Create toast container if it doesn't exist
    let toastContainer = document.querySelector('.toast-container');
    if (!toastContainer) {
        toastContainer = document.createElement('div');
        toastContainer.className = 'toast-container position-fixed bottom-0 end-0 p-3';
        document.body.appendChild(toastContainer);
    }

    // Create toast element
    const toastId = 'toast-' + Date.now();
    const toast = document.createElement('div');
    toast.className = `toast ${type === 'error' ? 'bg-danger text-white' : ''} ${type === 'success' ? 'bg-success text-white' : ''}`;
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'assertive');
    toast.setAttribute('aria-atomic', 'true');
    toast.setAttribute('id', toastId);

    toast.innerHTML = `
        <div class="toast-header">
            <strong class="me-auto">${type === 'error' ? 'Error' : 'Notification'}</strong>
            <button type="button" class="btn-close" data-bs-dismiss="toast" aria-label="Close"></button>
        </div>
        <div class="toast-body">
            ${message}
        </div>
    `;

    toastContainer.appendChild(toast);

    // Initialize and show the toast
    const toastElement = new bootstrap.Toast(toast, { delay: 5000 });
    toastElement.show();

    // Remove toast from DOM after it's hidden
    toast.addEventListener('hidden.bs.toast', () => {
        toast.remove();
    });
}

/**
 * Debounce function to limit the rate at which a function can fire
 */
function debounce(func, wait) {
    let timeout;
    return function (...args) {
        const context = this;
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(context, args), wait);
    };
}
