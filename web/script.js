// Configuration
const API_HOST = 'localhost';
const API_PORT = 8080;
const WS_HOST = 'localhost';
const WS_PORT = 8081;

// State
let currentUser = null;
let currentDocument = null;
let websocket = null;
let editor = null;
let documentList = [];
let collaborators = [];
let userColors = {};
let connected = false;

// DOM Elements
const loginBtn = document.getElementById('login-btn');
const usernameInput = document.getElementById('username-input');
const userIdElement = document.getElementById('user-id');
const newDocBtn = document.getElementById('new-doc-btn');
const saveBtn = document.getElementById('save-btn');
const compileBtn = document.getElementById('compile-btn');
const shareBtn = document.getElementById('share-btn');
const documentListElement = document.getElementById('document-list');
const userListElement = document.getElementById('user-list');
const connectionStatus = document.getElementById('connection-status');
const statusText = document.getElementById('status-text');

// Initialize Editor
document.addEventListener('DOMContentLoaded', () => {
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

    // Add editor change event listener
    editor.session.on('change', debounce(handleEditorChange, 500));

    // Initialize modals
    initializeModals();

    // Disable buttons until login
    toggleEditingControls(false);
});

// Initialize Bootstrap modals
function initializeModals() {
    const newDocModal = new bootstrap.Modal(document.getElementById('new-doc-modal'));
    const shareModal = new bootstrap.Modal(document.getElementById('share-modal'));

    // New Document button
    newDocBtn.addEventListener('click', () => {
        if (!currentUser) {
            alert('Please login first');
            return;
        }
        newDocModal.show();
    });

    // Create Document button
    document.getElementById('create-doc-btn').addEventListener('click', () => {
        const title = document.getElementById('doc-title').value;
        const repoUrl = document.getElementById('repo-url').value;

        if (title.trim() === '') {
            alert('Please enter a document title');
            return;
        }

        createDocument(title, repoUrl).then(() => {
            newDocModal.hide();
            document.getElementById('new-doc-form').reset();
        });
    });

    // Share button
    shareBtn.addEventListener('click', () => {
        if (!currentDocument) {
            alert('Please open a document first');
            return;
        }

        const shareUrl = `${window.location.origin}?doc=${currentDocument.id}`;
        document.getElementById('share-url').value = shareUrl;

        // Clear and populate collaborator list
        const collaboratorList = document.getElementById('collaborator-list');
        collaboratorList.innerHTML = '';

        currentDocument.collaborators.forEach(user => {
            const li = document.createElement('li');
            li.className = 'list-group-item d-flex justify-content-between align-items-center';
            li.innerHTML = `
                <span>${user}</span>
                <button class="btn btn-sm btn-danger remove-collaborator" data-user="${user}">Remove</button>
            `;
            collaboratorList.appendChild(li);
        });

        // Add event listeners to remove buttons
        document.querySelectorAll('.remove-collaborator').forEach(btn => {
            btn.addEventListener('click', () => {
                const user = btn.getAttribute('data-user');
                removeCollaborator(currentDocument.id, user).then(() => {
                    btn.parentElement.remove();
                });
            });
        });

        shareModal.show();
    });

    // Copy share URL button
    document.getElementById('copy-btn').addEventListener('click', () => {
        const shareUrl = document.getElementById('share-url');
        shareUrl.select();
        document.execCommand('copy');
        alert('URL copied to clipboard');
    });

    // Add collaborator button
    document.getElementById('add-collaborator-btn').addEventListener('click', () => {
        const username = document.getElementById('collaborator-input').value;

        if (username.trim() === '') {
            alert('Please enter a username');
            return;
        }

        addCollaborator(currentDocument.id, username).then(() => {
            document.getElementById('collaborator-input').value = '';

            // Add to the list
            const collaboratorList = document.getElementById('collaborator-list');
            const li = document.createElement('li');
            li.className = 'list-group-item d-flex justify-content-between align-items-center';
            li.innerHTML = `
                <span>${username}</span>
                <button class="btn btn-sm btn-danger remove-collaborator" data-user="${username}">Remove</button>
            `;
            collaboratorList.appendChild(li);

            // Add event listener to remove button
            li.querySelector('.remove-collaborator').addEventListener('click', () => {
                const user = li.querySelector('.remove-collaborator').getAttribute('data-user');
                removeCollaborator(currentDocument.id, user).then(() => {
                    li.remove();
                });
            });
        });
    });
}

// Login functionality
loginBtn.addEventListener('click', () => {
    const username = usernameInput.value.trim();
    if (username === '') {
        alert('Please enter a username');
        return;
    }

    currentUser = username;
    userIdElement.textContent = `User: ${username}`;

    // Connect to WebSocket
    connectWebSocket();

    // Get document list
    fetchDocuments();

    // Enable editing controls
    toggleEditingControls(true);
});

// Save button functionality
saveBtn.addEventListener('click', () => {
    if (!currentDocument) {
        alert('No document is currently open');
        return;
    }

    saveDocument();
});

// Compile button functionality
compileBtn.addEventListener('click', () => {
    if (!currentDocument) {
        alert('No document is currently open');
        return;
    }

    compileDocument();
});

// Connect to WebSocket
function connectWebSocket() {
    if (websocket) {
        websocket.close();
    }

    websocket = new WebSocket(`ws://${WS_HOST}:${WS_PORT}`);

    websocket.onopen = () => {
        console.log('WebSocket connection established');
        setConnectionStatus(true);

        // Authenticate
        sendWebSocketMessage({
            type: 'Authentication',
            payload: {
                user_id: currentUser,
                token: null // No authentication token for demo
            }
        });
    };

    websocket.onclose = () => {
        console.log('WebSocket connection closed');
        setConnectionStatus(false);

        // Attempt to reconnect after a delay
        setTimeout(connectWebSocket, 5000);
    };

    websocket.onerror = (error) => {
        console.error('WebSocket error:', error);
        setConnectionStatus(false);
    };

    websocket.onmessage = (event) => {
        handleWebSocketMessage(JSON.parse(event.data));
    };
}

// Set connection status indicator
function setConnectionStatus(isConnected) {
    connected = isConnected;
    connectionStatus.className = isConnected ? 'status-dot online' : 'status-dot offline';
    statusText.textContent = isConnected ? 'Connected' : 'Disconnected';
}

// Send message through WebSocket
function sendWebSocketMessage(message) {
    if (websocket && websocket.readyState === WebSocket.OPEN) {
        websocket.send(JSON.stringify(message));
    } else {
        console.warn('WebSocket not connected, message not sent:', message);
    }
}

// Handle incoming WebSocket messages
function handleWebSocketMessage(message) {
    console.log('Received WebSocket message:', message);

    switch (message.type) {
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
            console.warn('Unknown message type:', message.type);
    }
}

// Handle document update messages
function handleDocumentUpdate(payload) {
    const { document_id, content, version } = payload;

    // Only update if this is the current document
    if (currentDocument && currentDocument.id === document_id) {
        // Update editor content without triggering change event
        const cursorPosition = editor.getCursorPosition();
        editor.session.setValue(content);
        editor.moveCursorToPosition(cursorPosition);

        // Update preview if available
        updatePreview();
    }
}

// Handle document list messages
function handleDocumentList(payload) {
    documentList = payload.documents;
    updateDocumentList();
}

// Handle presence update messages
function handlePresenceUpdate(payload) {
    const { document_id, presence } = payload;

    // Only update if this is the current document
    if (currentDocument && currentDocument.id === document_id) {
        // Update collaborator list
        updateCollaboratorList(presence);
    }
}

// Handle error messages
function handleError(payload) {
    const { code, message } = payload;
    console.error(`Error ${code}: ${message}`);
    alert(`Error: ${message}`);
}

// Update document list in sidebar
function updateDocumentList() {
    documentListElement.innerHTML = '';

    documentList.forEach(doc => {
        const li = document.createElement('li');
        li.textContent = doc.title;
        li.dataset.id = doc.id;

        if (currentDocument && currentDocument.id === doc.id) {
            li.classList.add('active');
        }

        li.addEventListener('click', () => openDocument(doc.id));
        documentListElement.appendChild(li);
    });
}

// Update collaborator list
function updateCollaboratorList(presence) {
    userListElement.innerHTML = '';

    collaborators = Object.keys(presence);

    collaborators.forEach(userId => {
        if (!userColors[userId]) {
            userColors[userId] = getRandomColor();
        }

        const li = document.createElement('li');
        li.innerHTML = `
            <div class="user-badge" style="background-color: ${userColors[userId]};"></div>
            ${userId}${userId === currentUser ? ' (You)' : ''}
        `;

        userListElement.appendChild(li);
    });
}

// Handle editor changes
function handleEditorChange(delta) {
    if (!currentDocument || !connected) return;

    // Convert ACE delta to CRDT operation
    const operation = convertDeltaToOperation(delta);
    if (!operation) return;

    // Send operation to server
    sendWebSocketMessage({
        type: 'DocumentOperation',
        payload: {
            operation: {
                ...operation,
                document_id: currentDocument.id
            }
        }
    });
}

// Convert ACE delta to CRDT operation
function convertDeltaToOperation(delta) {
    const action = delta.action;
    const start = delta.start;
    const end = delta.end;
    const lines = delta.lines;

    // Calculate the position in the document
    const position = getPositionInDocument(start);

    if (action === 'insert') {
        const content = lines.join('\n');
        return {
            type: 'Insert',
            position,
            content
        };
    } else if (action === 'remove') {
        const endPosition = getPositionInDocument(end);
        return {
            type: 'Delete',
            range: {
                start: position,
                end: endPosition
            }
        };
    }

    return null;
}

// Get absolute position in document from ACE position
function getPositionInDocument(acePosition) {
    const doc = editor.session.getDocument();
    let position = 0;

    for (let i = 0; i < acePosition.row; i++) {
        position += doc.getLine(i).length + 1; // +1 for the newline
    }

    position += acePosition.column;
    return position;
}

// Fetch document list from API
async function fetchDocuments() {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/documents`);
        const data = await response.json();

        documentList = data.documents;
        updateDocumentList();
    } catch (error) {
        console.error('Error fetching documents:', error);
        alert('Failed to fetch document list');
    }
}

// Create a new document
async function createDocument(title, repoUrl) {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/documents`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                title,
                owner: currentUser,
                repository_url: repoUrl || null
            })
        });

        const data = await response.json();

        // Refresh document list
        fetchDocuments();

        // Open the new document
        openDocument(data.document_id);
    } catch (error) {
        console.error('Error creating document:', error);
        alert('Failed to create document');
    }
}

// Open a document
async function openDocument(docId) {
    try {
        // First, get document metadata
        const response = await fetch(`http://${API_HOST}:${API_PORT}/documents/${docId}`);
        const document = await response.json();

        currentDocument = document;

        // Update active document in list
        document.querySelectorAll('#document-list li').forEach(li => {
            li.classList.remove('active');
            if (li.dataset.id === docId) {
                li.classList.add('active');
            }
        });

        // Request document content through WebSocket
        sendWebSocketMessage({
            type: 'OpenDocument',
            payload: {
                document_id: docId
            }
        });
    } catch (error) {
        console.error('Error opening document:', error);
        alert('Failed to open document');
    }
}

// Save document
async function saveDocument() {
    try {
        // Use Git sync endpoint to save the document
        const response = await fetch(`http://${API_HOST}:${API_PORT}/documents/${currentDocument.id}/sync`, {
            method: 'POST'
        });

        if (response.ok) {
            alert('Document saved successfully');
        } else {
            throw new Error('Failed to save document');
        }
    } catch (error) {
        console.error('Error saving document:', error);
        alert('Failed to save document');
    }
}

// Compile document
async function compileDocument() {
    try {
        // For the demo, we'll just simulate compilation by updating the preview
        updatePreview();
        alert('Document compiled successfully');
    } catch (error) {
        console.error('Error compiling document:', error);
        alert('Failed to compile document');
    }
}

// Update preview pane
function updatePreview() {
    const preview = document.getElementById('preview');

    // In a real implementation, this would render LaTeX to HTML or PDF
    // For the demo, we'll just format it a bit
    const content = editor.getValue();
    const formatted = formatLaTeXForPreview(content);
    preview.innerHTML = formatted;
}

// Format LaTeX for preview display
function formatLaTeXForPreview(latex) {
    // Extract title
    const titleMatch = latex.match(/\\title\{(.*?)\}/);
    const title = titleMatch ? titleMatch[1] : 'LaTeX Document';

    // Extract sections
    const sections = [];
    const sectionRegex = /\\section\{(.*?)\}([\s\S]*?)(?=\\section\{|\\end\{document\})/g;
    let match;
    while ((match = sectionRegex.exec(latex)) !== null) {
        sections.push({
            title: match[1],
            content: match[2].trim()
        });
    }

    // Build HTML
    let html = `<h1>${title}</h1>`;

    sections.forEach(section => {
        html += `<h2>${section.title}</h2>`;
        html += `<div>${formatContent(section.content)}</div>`;
    });

    return html;
}

// Format LaTeX content for preview
function formatContent(content) {
    // Replace equations
    content = content.replace(/\\begin\{equation\}([\s\S]*?)\\end\{equation\}/g,
        '<div class="equation">$1</div>');

    // Replace itemize lists
    content = content.replace(/\\begin\{itemize\}([\s\S]*?)\\end\{itemize\}/g,
        (match, listContent) => {
            const items = listContent.match(/\\item (.*?)(?=\\item|$)/g) || [];
            let html = '<ul>';
            items.forEach(item => {
                html += `<li>${item.replace('\\item ', '')}</li>`;
            });
            html += '</ul>';
            return html;
        });

    // Replace paragraph breaks
    content = content.replace(/\n\n/g, '<br><br>');

    return content;
}

// Add collaborator
async function addCollaborator(docId, username) {
    // This would normally be an API call
    // For the demo, we'll simulate it
    return new Promise(resolve => {
        setTimeout(() => {
            currentDocument.collaborators.push(username);
            resolve();
        }, 500);
    });
}

// Remove collaborator
async function removeCollaborator(docId, username) {
    // This would normally be an API call
    // For the demo, we'll simulate it
    return new Promise(resolve => {
        setTimeout(() => {
            const index = currentDocument.collaborators.indexOf(username);
            if (index > -1) {
                currentDocument.collaborators.splice(index, 1);
            }
            resolve();
        }, 500);
    });
}

// Toggle editing controls
function toggleEditingControls(enabled) {
    newDocBtn.disabled = !enabled;
    saveBtn.disabled = !enabled;
    compileBtn.disabled = !enabled;
    shareBtn.disabled = !enabled;
    editor.setReadOnly(!enabled);
}

// Get random color for user badge
function getRandomColor() {
    const colors = [
        '#1abc9c', '#2ecc71', '#3498db', '#9b59b6', '#34495e',
        '#16a085', '#27ae60', '#2980b9', '#8e44ad', '#2c3e50',
        '#f1c40f', '#e67e22', '#e74c3c', '#ecf0f1', '#95a5a6',
        '#f39c12', '#d35400', '#c0392b', '#bdc3c7', '#7f8c8d'
    ];
    return colors[Math.floor(Math.random() * colors.length)];
}

// Debounce function for editor changes
function debounce(func, wait) {
    let timeout;
    return function (...args) {
        const context = this;
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(context, args), wait);
    };
}
