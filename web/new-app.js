/**
 * TeXSwarm - Collaborative LaTeX Editor
 * This is the main JavaScript file for the application.
 */

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
let websocket = null;
let connectionTimeout = null;
let heartbeatTimeout = null;
let heartbeatInterval = null;
let lastHeartbeat = null;

/**
 * Initialize the application when the DOM is loaded
 */
document.addEventListener('DOMContentLoaded', () => {
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
        const response = await fetch(`${API_URL}/api/ping`);

        if (response.ok) {
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
        websocket = new WebSocket(WS_URL);

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
 * Handle WebSocket message event
 */
function handleWebSocketMessage(event) {
    try {
        const message = JSON.parse(event.data);
        console.log('Received message:', message);

        // Process different message types
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
    } catch (error) {
        console.error('Error parsing WebSocket message:', error);
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
            throw new Error('Failed to create document');
        }

        const data = await response.json();

        // Request updated document list
        requestDocumentList();

        // Open the new document
        openDocument(data.document_id);

        showToast(`Document "${title}" created`);
    } catch (error) {
        console.error('Document creation error:', error);
        showToast('Failed to create document', 'error');
    }
}

/**
 * Open a document
 */
async function openDocument(documentId) {
    if (!currentUser) {
        showToast('Please login first', 'error');
        return;
    }

    try {
        // Send request to get document
        const response = await fetch(`${API_URL}/api/documents/${documentId}`, {
            headers: {
                'x-user-id': currentUser.id
            }
        });

        if (!response.ok) {
            throw new Error('Failed to open document');
        }

        const doc = await response.json();

        // Set as current document
        currentDocument = {
            id: doc.id,
            title: doc.title,
            owner: doc.owner,
            version: doc.version
        };

        // Update UI
        document.getElementById('document-title').textContent = doc.title;
        editor.session.setValue(doc.content || '');
        updateDocumentList(); // To highlight active document

        // Send WebSocket message to open document
        if (isWebSocketConnected()) {
            const message = {
                type: 'OpenDocument',
                payload: {
                    document_id: documentId
                }
            };

            websocket.send(JSON.stringify(message));
        }

        showToast(`Document "${doc.title}" opened`);
    } catch (error) {
        console.error('Document opening error:', error);
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
