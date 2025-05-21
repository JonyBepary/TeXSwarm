// Configuration
const API_HOST = window.location.hostname || 'localhost';
const API_PORT = 8090;
const WS_HOST = window.location.hostname || 'localhost';
const WS_PORT = 8091;

// State
let currentUser = null;
let currentDocument = null;
let websocket = null;
let editor = null;
let documentList = [];
let collaborators = [];
let userColors = {};
let connected = false;
let loginModal = null;

// Initialize on document load
document.addEventListener('DOMContentLoaded', () => {
    // Initialize Editor
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

    // Check for URL parameters to handle document sharing
    handleUrlParameters();

    // Show login modal
    loginModal = new bootstrap.Modal(document.getElementById('login-modal'));
    loginModal.show();

    // Setup event listeners
    setupEventListeners();

    // Update template preview when template is selected
    document.getElementById('template-select').addEventListener('change', updateTemplatePreview);
});

// Initialize Bootstrap modals
function initializeModals() {
    const newDocModal = new bootstrap.Modal(document.getElementById('new-doc-modal'));
    const shareModal = new bootstrap.Modal(document.getElementById('share-modal'));

    // New Document button
    document.getElementById('new-doc-btn').addEventListener('click', () => {
        if (!currentUser) {
            alert('Please log in first');
            return;
        }
        document.getElementById('doc-title').value = "";
        document.getElementById('doc-description').value = "";
        document.getElementById('template-title').value = "";
        document.getElementById('repo-url').value = "";
        document.getElementById('repo-branch').value = "";
        document.getElementById('repo-path').value = "";
        newDocModal.show();
    });

    // Create Document button
    document.getElementById('create-doc-btn').addEventListener('click', () => {
        const activeTab = document.querySelector('.tab-pane.active');
        const tabId = activeTab.id;

        let title, repoUrl, content = null;

        if (tabId === 'blank-doc') {
            title = document.getElementById('doc-title').value;
            if (title.trim() === '') {
                alert('Please enter a document title');
                return;
            }
            content = getDefaultLatexTemplate(title, currentUser.name);
        }
        else if (tabId === 'template-doc') {
            title = document.getElementById('template-title').value;
            if (title.trim() === '') {
                alert('Please enter a document title');
                return;
            }
            const templateType = document.getElementById('template-select').value;
            content = getLatexTemplate(templateType, title, currentUser.name);
        }
        else if (tabId === 'git-doc') {
            repoUrl = document.getElementById('repo-url').value;
            if (repoUrl.trim() === '') {
                alert('Please enter a Git repository URL');
                return;
            }
            const branch = document.getElementById('repo-branch').value;
            const path = document.getElementById('repo-path').value;

            title = repoUrl.split('/').pop().replace('.git', '');
            createDocumentFromGit(title, repoUrl, branch, path).then(() => {
                newDocModal.hide();
            });
            return;
        }

        createDocument(title, content).then(() => {
            newDocModal.hide();
        });
    });

    // Share button
    document.getElementById('share-btn').addEventListener('click', () => {
        if (!currentDocument) {
            alert('Please open a document first');
            return;
        }

        const shareUrl = `${window.location.origin}?doc=${currentDocument.id}`;
        document.getElementById('share-url').value = shareUrl;

        // Clear and populate collaborator list
        updateCollaboratorListModal();

        shareModal.show();
    });

    // Copy URL button
    document.getElementById('copy-url-btn').addEventListener('click', () => {
        const shareUrl = document.getElementById('share-url');
        shareUrl.select();
        navigator.clipboard.writeText(shareUrl.value);

        // Change button text temporarily
        const copyBtn = document.getElementById('copy-url-btn');
        const originalContent = copyBtn.innerHTML;
        copyBtn.innerHTML = '<i class="bi bi-check"></i>';
        setTimeout(() => {
            copyBtn.innerHTML = originalContent;
        }, 2000);
    });
}

function setupEventListeners() {
    // Login form submission
    document.getElementById('login-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const username = document.getElementById('username-input').value;
        if (username.trim() === '') return;

        login(username);
    });

    // Logout button
    document.getElementById('logout-btn').addEventListener('click', () => {
        logout();
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
}

// Login user
async function login(username) {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/users`, {
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
        currentUser = {
            id: data.id,
            name: username
        };

        document.getElementById('user-name').textContent = username;
        document.getElementById('user-avatar').textContent = getInitials(username);

        // Hide login modal and show main app
        loginModal.hide();
        document.querySelector('.app-container').classList.remove('d-none');

        // Connect websocket
        connectWebSocket();

        // Load documents
        loadDocuments();

        // Enable editing controls
        toggleEditingControls(true);

        console.log('Logged in successfully:', currentUser);
    } catch (error) {
        console.error('Login error:', error);
        alert('Failed to login. Please try again.');
    }
}

function logout() {
    // Disconnect websocket
    if (websocket) {
        websocket.close();
    }

    // Reset state
    currentUser = null;
    currentDocument = null;
    documentList = [];
    collaborators = [];

    // Update UI
    document.getElementById('user-name').textContent = 'Not logged in';
    document.getElementById('user-avatar').textContent = '';
    document.getElementById('status-text').textContent = 'Disconnected';
    document.getElementById('status-dot').classList.remove('connected');
    document.getElementById('document-list').innerHTML = '<li class="text-center text-muted py-4">No documents yet</li>';
    document.getElementById('collaborator-list').innerHTML = '<div class="text-muted small">No active collaborators</div>';
    document.getElementById('current-doc-title').textContent = 'No document opened';
    editor.setValue('');
    document.getElementById('preview').innerHTML = '<div class="text-center text-muted py-5"><i class="bi bi-file-earmark-text" style="font-size: 3rem;"></i><p class="mt-3">Your LaTeX preview will appear here</p></div>';

    // Disable editing controls
    toggleEditingControls(false);

    // Hide main app and show login
    document.querySelector('.app-container').classList.add('d-none');
    document.getElementById('username-input').value = '';
    loginModal.show();
}

// Connect to WebSocket server
function connectWebSocket() {
    if (websocket) {
        websocket.close();
    }

    // Update UI to show connecting status
    const statusText = document.getElementById('status-text');
    statusText.textContent = 'Connecting...';
    console.log(`Connecting to WebSocket server at ws://${WS_HOST}:${WS_PORT}/ws`);

    try {
        // Create new WebSocket connection
        console.log('Creating new WebSocket connection...');
        websocket = new WebSocket(`ws://${WS_HOST}:${WS_PORT}/ws`);
        console.log('WebSocket object created:', websocket);

        let reconnectAttempts = 0;
        const maxReconnectAttempts = 5;
        const baseReconnectDelay = 2000; // 2 seconds

        websocket.onopen = () => {
            console.log('WebSocket connected successfully');
            connected = true;
            reconnectAttempts = 0; // Reset reconnect attempts on successful connection
            updateConnectionStatus(true);

            // If we have an active document, resubscribe
            if (currentDocument && currentUser) {
                const subscribeMessage = {
                    type: 'subscribe',
                    document_id: currentDocument.id
                };
                websocket.send(JSON.stringify(subscribeMessage));
            }

            // Authenticate via WebSocket
            if (currentUser) {
                const authMessage = {
                    type: 'auth',
                    user_id: currentUser.id
                };
                websocket.send(JSON.stringify(authMessage));
            }
        };

        websocket.onclose = (event) => {
            console.log(`WebSocket disconnected: Code ${event.code}${event.reason ? ', Reason: ' + event.reason : ''}`);
            connected = false;
            updateConnectionStatus(false);

            // Exponential backoff for reconnection
            if (currentUser && reconnectAttempts < maxReconnectAttempts) {
                const delay = baseReconnectDelay * Math.pow(1.5, reconnectAttempts);
                reconnectAttempts++;

                console.log(`Attempting to reconnect (${reconnectAttempts}/${maxReconnectAttempts}) in ${delay / 1000} seconds...`);
                statusText.textContent = `Reconnecting (${reconnectAttempts}/${maxReconnectAttempts})...`;

                setTimeout(() => {
                    if (!connected && currentUser) {
                        connectWebSocket();
                    }
                }, delay);
            } else if (reconnectAttempts >= maxReconnectAttempts) {
                statusText.textContent = 'Connection failed';
                console.error('Maximum reconnection attempts reached. Please check server status.');
                alert('Could not connect to server after multiple attempts. Please check your connection and try again.');
            }
        };

        websocket.onerror = (error) => {
            console.error('WebSocket error:', error);
            connected = false;
            updateConnectionStatus(false);
        };

        websocket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                handleWebSocketMessage(message);
            } catch (error) {
                console.error('Error parsing WebSocket message:', error);
            }
        };
    }
} // Close the connectWebSocket function

// Handle WebSocket messages
function handleWebSocketMessage(message) {
    console.log('Received WebSocket message:', message);

    switch (message.type) {
        case 'document_update':
            if (currentDocument && message.document_id === currentDocument.id) {
                handleDocumentUpdate(message);
            }
            break;

        case 'user_presence':
            if (currentDocument && message.document_id === currentDocument.id) {
                updateCollaboratorPresence(message);
            }
            break;

        case 'compile_result':
            if (currentDocument && message.document_id === currentDocument.id) {
                updatePreview(message.content);
            }
            break;

        case 'error':
            console.error('Server error:', message.message);
            alert(`Server error: ${message.message}`);
            break;
    }
}

// Handle document updates from other users
function handleDocumentUpdate(message) {
    if (!currentDocument || message.document_id !== currentDocument.id) return;

    // Check if update is from someone else
    if (message.user_id !== currentUser.id) {
        // Apply the changes to the editor without triggering our own change event
        const currentPosition = editor.getCursorPosition();

        // Temporarily remove our change listener
        const session = editor.getSession();
        const changeListeners = session._eventRegistry.change;
        session._eventRegistry.change = [];

        editor.setValue(message.content);
        editor.gotoLine(currentPosition.row + 1, currentPosition.column);

        // Restore change listeners
        session._eventRegistry.change = changeListeners;
    }
}

// Update collaborator presence information
function updateCollaboratorPresence(message) {
    // Update or add the collaborator
    let collaborator = collaborators.find(c => c.userId === message.user_id);

    if (!collaborator) {
        // New collaborator
        collaborator = {
            userId: message.user_id,
            name: message.user_name,
            isActive: message.is_active,
            lastActivity: new Date(),
            color: getRandomColor(message.user_id)
        };
        collaborators.push(collaborator);
    } else {
        // Update existing collaborator
        collaborator.isActive = message.is_active;
        collaborator.lastActivity = new Date();
        if (message.cursor_position !== undefined) {
            collaborator.cursorPosition = message.cursor_position;
        }
    }

    // Update the UI
    updateCollaboratorList();
}

// Update the collaborator list in the sidebar
function updateCollaboratorList() {
    const container = document.getElementById('collaborator-list');

    if (collaborators.length === 0) {
        container.innerHTML = '<div class="text-muted small">No active collaborators</div>';
        return;
    }

    // Sort collaborators by activity (active first, then by name)
    const sortedCollaborators = [...collaborators].sort((a, b) => {
        if (a.isActive !== b.isActive) {
            return b.isActive - a.isActive; // Active users first
        }
        return a.name.localeCompare(b.name);
    });

    container.innerHTML = '';
    sortedCollaborators.forEach(collab => {
        // Skip the current user
        if (currentUser && collab.userId === currentUser.id) return;

        const item = document.createElement('div');
        item.className = 'collaborator-item';

        const badge = document.createElement('div');
        badge.className = 'user-badge';
        badge.style.backgroundColor = collab.color;

        const nameSpan = document.createElement('span');
        nameSpan.textContent = collab.name;
        nameSpan.style.opacity = collab.isActive ? 1 : 0.5;

        item.appendChild(badge);
        item.appendChild(nameSpan);
        container.appendChild(item);
    });
}

// Update collaborator list in the share modal
function updateCollaboratorListModal() {
    const container = document.getElementById('collaborator-list-modal');

    if (collaborators.length === 0) {
        container.innerHTML = '<li class="list-group-item text-center text-muted">No collaborators yet</li>';
        return;
    }

    container.innerHTML = '';
    collaborators.forEach(collab => {
        const li = document.createElement('li');
        li.className = 'list-group-item d-flex justify-content-between align-items-center';

        const userSpan = document.createElement('span');
        userSpan.innerHTML = `<span class="user-badge" style="background-color: ${collab.color}"></span> ${collab.name}`;

        const statusBadge = document.createElement('span');
        statusBadge.className = `badge ${collab.isActive ? 'bg-success' : 'bg-secondary'}`;
        statusBadge.textContent = collab.isActive ? 'Active' : 'Inactive';

        li.appendChild(userSpan);
        li.appendChild(statusBadge);
        container.appendChild(li);
    });
}

// Load user's documents
async function loadDocuments() {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents`, {
            headers: {
                'X-User-ID': currentUser.id
            }
        });

        if (!response.ok) {
            throw new Error('Failed to load documents');
        }

        const data = await response.json();
        documentList = data;
        updateDocumentList();
    } catch (error) {
        console.error('Error loading documents:', error);
    }
}

// Update the document list in the sidebar
function updateDocumentList() {
    const documentListElement = document.getElementById('document-list');

    if (documentList.length === 0) {
        documentListElement.innerHTML = '<li class="text-center text-muted py-4">No documents yet</li>';
        return;
    }

    documentListElement.innerHTML = '';
    documentList.forEach(doc => {
        const li = document.createElement('li');
        if (currentDocument && doc.id === currentDocument.id) {
            li.className = 'active';
        }

        li.innerHTML = `<i class="bi bi-file-earmark-text"></i> ${doc.title}`;
        li.addEventListener('click', () => openDocument(doc.id));
        documentListElement.appendChild(li);
    });
}

// Create a new document
async function createDocument(title, content = null) {
    try {
        const documentData = {
            title: title,
            owner_id: currentUser.id
        };

        if (content) {
            documentData.content = content;
        }

        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-User-ID': currentUser.id
            },
            body: JSON.stringify(documentData)
        });

        if (!response.ok) {
            throw new Error('Failed to create document');
        }

        const data = await response.json();
        console.log('Document created:', data);

        // Add to document list and open it
        documentList.push(data);
        updateDocumentList();
        openDocument(data.id);

        return data;
    } catch (error) {
        console.error('Error creating document:', error);
        alert('Failed to create document. Please try again.');
    }
}

// Create a document from Git repository
async function createDocumentFromGit(title, repoUrl, branch = '', path = '') {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents/git`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-User-ID': currentUser.id
            },
            body: JSON.stringify({
                title: title,
                repo_url: repoUrl,
                branch: branch || undefined,
                file_path: path || undefined,
                owner_id: currentUser.id
            })
        });

        if (!response.ok) {
            throw new Error('Failed to create document from Git');
        }

        const data = await response.json();
        console.log('Document created from Git:', data);

        // Add to document list and open it
        documentList.push(data);
        updateDocumentList();
        openDocument(data.id);

        return data;
    } catch (error) {
        console.error('Error creating document from Git:', error);
        alert('Failed to create document from Git. Please try again.');
    }
}

// Open a document
async function openDocument(docId) {
    try {
        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents/${docId}`, {
            headers: {
                'X-User-ID': currentUser.id
            }
        });

        if (!response.ok) {
            throw new Error('Failed to open document');
        }

        const data = await response.json();
        currentDocument = data;

        // Update UI
        document.getElementById('current-doc-title').textContent = data.title;
        editor.setValue(data.content || '');
        editor.clearSelection();

        // Subscribe to document events via WebSocket
        if (websocket && websocket.readyState === WebSocket.OPEN) {
            const subscribeMessage = {
                type: 'subscribe',
                document_id: docId,
                user_id: currentUser.id
            };
            websocket.send(JSON.stringify(subscribeMessage));
        }

        // Update document list selection
        updateDocumentList();

        // Enable editing controls
        toggleEditingControls(true);

        // Compile the document to update preview
        compileDocument();
    } catch (error) {
        console.error('Error opening document:', error);
        alert('Failed to open document. Please try again.');
    }
}

// Save the current document
async function saveDocument() {
    if (!currentDocument) return;

    try {
        const content = editor.getValue();

        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents/${currentDocument.id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'X-User-ID': currentUser.id
            },
            body: JSON.stringify({
                content: content
            })
        });

        if (!response.ok) {
            throw new Error('Failed to save document');
        }

        console.log('Document saved successfully');

        // Show a brief success message
        const saveBtn = document.getElementById('save-btn');
        const originalContent = saveBtn.innerHTML;
        saveBtn.innerHTML = '<i class="bi bi-check"></i> Saved';
        saveBtn.disabled = true;
        setTimeout(() => {
            saveBtn.innerHTML = originalContent;
            saveBtn.disabled = false;
        }, 2000);
    } catch (error) {
        console.error('Error saving document:', error);
        alert('Failed to save document. Please try again.');
    }
}

// Compile the current document
async function compileDocument() {
    if (!currentDocument) return;

    try {
        const content = editor.getValue();

        const response = await fetch(`http://${API_HOST}:${API_PORT}/api/documents/${currentDocument.id}/compile`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-User-ID': currentUser.id
            },
            body: JSON.stringify({
                content: content
            })
        });

        if (!response.ok) {
            throw new Error('Failed to compile document');
        }

        const data = await response.json();
        updatePreview(data.html || data.content);
    } catch (error) {
        console.error('Error compiling document:', error);
        document.getElementById('preview').innerHTML = `
            <div class="alert alert-danger" role="alert">
                <h4 class="alert-heading">Compilation Error</h4>
                <p>${error.message || 'Failed to compile document. Please check your LaTeX syntax.'}</p>
            </div>
        `;
    }
}

// Update the preview with compiled content
function updatePreview(html) {
    const preview = document.getElementById('preview');
    preview.innerHTML = html;

    // Render math formulas
    if (window.MathJax) {
        window.MathJax.typeset();
    }
}

// Handle editor changes
function handleEditorChange() {
    if (!currentDocument || !currentUser || !websocket) return;

    const content = editor.getValue();

    // Send document update to server
    if (websocket.readyState === WebSocket.OPEN) {
        const updateMessage = {
            type: 'document_update',
            document_id: currentDocument.id,
            user_id: currentUser.id,
            content: content
        };
        websocket.send(JSON.stringify(updateMessage));
    }

    // Update cursor position
    const cursor = editor.getCursorPosition();
    sendCursorPosition(cursor.row, cursor.column);
}

// Send cursor position update
function sendCursorPosition(row, column) {
    if (!currentDocument || !currentUser || !websocket) return;

    if (websocket.readyState === WebSocket.OPEN) {
        const presenceMessage = {
            type: 'user_presence',
            document_id: currentDocument.id,
            user_id: currentUser.id,
            user_name: currentUser.name,
            cursor_position: {
                row: row,
                column: column
            },
            is_active: true
        };
        websocket.send(JSON.stringify(presenceMessage));
    }
}

// Update connection status in the UI
function updateConnectionStatus(isConnected) {
    const statusDot = document.getElementById('status-dot');
    const statusText = document.getElementById('status-text');

    if (isConnected) {
        statusDot.classList.add('connected');
        statusText.textContent = 'Connected';
    } else {
        statusDot.classList.remove('connected');
        statusText.textContent = 'Disconnected';
    }
}

// Toggle editing controls based on login state
function toggleEditingControls(enabled) {
    document.getElementById('save-btn').disabled = !enabled || !currentDocument;
    document.getElementById('compile-btn').disabled = !enabled || !currentDocument;
    document.getElementById('share-btn').disabled = !enabled || !currentDocument;

    if (editor) {
        editor.setReadOnly(!enabled);
    }
}

// Handle URL parameters for document sharing
function handleUrlParameters() {
    const urlParams = new URLSearchParams(window.location.search);
    const docId = urlParams.get('doc');

    if (docId) {
        // Store the document ID to open after login
        window.documentToOpen = docId;
    }
}

// Update the template preview when a template is selected
function updateTemplatePreview() {
    const template = document.getElementById('template-select').value;
    const preview = document.getElementById('latex-template');

    switch (template) {
        case 'article':
            preview.innerHTML = `\\documentclass{article}<br>
\\title{Document Title}<br>
\\author{Your Name}<br>
\\date{\\today}<br><br>
\\begin{document}<br><br>
\\maketitle<br><br>
\\section{Introduction}<br>
Your content here...<br><br>
\\end{document}`;
            break;
        case 'report':
            preview.innerHTML = `\\documentclass{report}<br>
\\title{Report Title}<br>
\\author{Your Name}<br>
\\date{\\today}<br><br>
\\begin{document}<br><br>
\\maketitle<br>
\\tableofcontents<br><br>
\\chapter{Introduction}<br>
Your content here...<br><br>
\\end{document}`;
            break;
        case 'book':
            preview.innerHTML = `\\documentclass{book}<br>
\\title{Book Title}<br>
\\author{Your Name}<br>
\\date{\\today}<br><br>
\\begin{document}<br><br>
\\frontmatter<br>
\\maketitle<br>
\\tableofcontents<br><br>
\\mainmatter<br>
\\chapter{Introduction}<br>
Your content here...<br><br>
\\end{document}`;
            break;
        case 'letter':
            preview.innerHTML = `\\documentclass{letter}<br>
\\usepackage{hyperref}<br>
\\signature{Your Name}<br>
\\address{Your Address\\\\City, State ZIP}<br><br>
\\begin{document}<br><br>
\\begin{letter}{Recipient Name\\\\Recipient Address\\\\City, State ZIP}<br><br>
\\opening{Dear Sir or Madam,}<br><br>
Letter content goes here...<br><br>
\\closing{Sincerely,}<br><br>
\\end{letter}<br>
\\end{document}`;
            break;
        case 'presentation':
            preview.innerHTML = `\\documentclass{beamer}<br>
\\usepackage{graphicx}<br>
\\title{Presentation Title}<br>
\\author{Your Name}<br>
\\institute{Your Institution}<br>
\\date{\\today}<br><br>
\\begin{document}<br><br>
\\frame{\\titlepage}<br><br>
\\begin{frame}<br>
\\frametitle{Introduction}<br>
\\begin{itemize}<br>
\\item First point<br>
\\item Second point<br>
\\item Third point<br>
\\end{itemize}<br>
\\end{frame}<br><br>
\\end{document}`;
            break;
    }
}

// Get the default LaTeX template
function getDefaultLatexTemplate(title, author) {
    return `\\documentclass{article}
\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\maketitle

\\section{Introduction}
Start your document here...

\\end{document}`;
}

// Get a specific LaTeX template
function getLatexTemplate(templateType, title, author) {
    switch (templateType) {
        case 'article':
            return getDefaultLatexTemplate(title, author);
        case 'report':
            return `\\documentclass{report}
\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\maketitle
\\tableofcontents

\\chapter{Introduction}
Start your report here...

\\chapter{Methods}
Describe your methods...

\\chapter{Results}
Present your results...

\\chapter{Discussion}
Discuss your findings...

\\end{document}`;
        case 'book':
            return `\\documentclass{book}
\\title{${title}}
\\author{${author}}
\\date{\\today}

\\begin{document}

\\frontmatter
\\maketitle
\\tableofcontents

\\mainmatter
\\chapter{Introduction}
Start your book here...

\\chapter{First Chapter}
Content of first chapter...

\\chapter{Second Chapter}
Content of second chapter...

\\appendix
\\chapter{Appendix}
Additional materials...

\\backmatter
\\begin{thebibliography}{9}
\\bibitem{key1} First reference
\\bibitem{key2} Second reference
\\end{thebibliography}

\\end{document}`;
        case 'letter':
            return `\\documentclass{letter}
\\usepackage{hyperref}
\\signature{${author}}
\\address{Your Address\\\\City, State ZIP}

\\begin{document}

\\begin{letter}{Recipient Name\\\\Recipient Address\\\\City, State ZIP}

\\opening{Dear Sir or Madam,}

This letter concerns...

\\closing{Sincerely,}

\\end{letter}
\\end{document}`;
        case 'presentation':
            return `\\documentclass{beamer}
\\usepackage{graphicx}
\\title{${title}}
\\author{${author}}
\\institute{Your Institution}
\\date{\\today}

\\begin{document}

\\frame{\\titlepage}

\\begin{frame}
\\frametitle{Outline}
\\tableofcontents
\\end{frame}

\\section{Introduction}
\\begin{frame}
\\frametitle{Introduction}
\\begin{itemize}
\\item First point
\\item Second point
\\item Third point
\\end{itemize}
\\end{frame}

\\section{Methods}
\\begin{frame}
\\frametitle{Methods}
Our approach includes...
\\end{frame}

\\section{Results}
\\begin{frame}
\\frametitle{Results}
We found that...
\\end{frame}

\\section{Conclusion}
\\begin{frame}
\\frametitle{Conclusion}
In conclusion...
\\end{frame}

\\end{document}`;
        default:
            return getDefaultLatexTemplate(title, author);
    }
}

// Utility Functions

// Generate random color based on user ID
function getRandomColor(userId) {
    if (userColors[userId]) {
        return userColors[userId];
    }

    // Generate a color based on the user ID
    const hash = Array.from(userId).reduce((acc, char) => {
        return char.charCodeAt(0) + ((acc << 5) - acc);
    }, 0);

    const h = Math.abs(hash) % 360;
    // Use a high saturation and lightness for vibrant but not too dark colors
    const color = `hsl(${h}, 70%, 65%)`;
    userColors[userId] = color;

    return color;
}

// Get user initials
function getInitials(name) {
    return name
        .split(' ')
        .map(part => part.charAt(0))
        .join('')
        .toUpperCase()
        .substring(0, 2);
}

// Debounce function to limit how often a function is called
function debounce(func, wait) {
    let timeout;
    return function (...args) {
        const context = this;
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(context, args), wait);
    };
}
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
    < span > ${ username }</span >
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
    userIdElement.textContent = `User: ${ username } `;

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
