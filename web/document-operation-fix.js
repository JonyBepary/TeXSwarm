/**
 * TeXSwarm - Collaborative LaTeX Editor
 * Document Operation Protocol Fix
 *
 * This script fixes the document operation format issues between the client and server.
 *
 * Problem 1: The client is using incorrect operation format with 'type' and 'user_id' fields
 * that don't match the expected server format of 'Insert', 'Delete', 'Replace'.
 *
 * Problem 2: The server expects document operations to be structured according to the
 * Operation enum in the Rust API, but the client is sending differently structured operations.
 *
 * Solution: This script intercepts WebSocket messages before they're sent and transforms
 * document operations to match the server's expected format.
 */

// Load this script after websocket-fix.js to patch document operations

(function () {
    // Keep a reference to the original WebSocket methods (in case they weren't overridden)
    const originalSend = WebSocket.prototype.send;
    const originalOnMessage = WebSocket.prototype.onmessage;

    // Document tracking state
    let documentTracker = {
        // Set of known document IDs
        knownDocuments: new Set(),
        // Store the last message that was a document operation for potential retry
        lastDocOpMessage: null,
        // Flag to indicate if we're in retry mode
        isRetrying: false,
        // Add a document ID to tracking
        trackDocument: function (docId) {
            if (docId) {
                this.knownDocuments.add(docId);
                console.log('Document operation fix: Now tracking document ID', docId);
            }
        },
        // Check if a document is known
        isKnownDocument: function (docId) {
            return docId ? this.knownDocuments.has(docId) : false;
        }
    };

    // Track pending document creations
    let pendingBranchCreations = {};
    let documentBranchStatus = {};
    let maxRetries = 3;

    // Function to check if a message is an error about missing document branches
    function isDocumentBranchError(message) {
        return message &&
            message.type === 'Error' &&
            message.payload &&
            message.payload.message &&
            message.payload.message.includes('Document branch not found');
    }

    // Function to extract document ID from an error message
    function extractDocumentIdFromError(message) {
        if (!message.payload || !message.payload.message) return null;

        const errorMsg = message.payload.message;
        const match = errorMsg.match(/Document branch not found: ([0-9a-f-]{36})/);
        return match ? match[1] : null;
    }

    // Function to fix a document branch issue
    function fixDocumentBranch(ws, documentId) {
        if (!documentId) return false;

        console.log(`Attempting to fix document branch for ${documentId}`);

        // Check if we're already trying to fix this document
        if (pendingBranchCreations[documentId]) {
            console.log(`Already attempting to fix document branch for ${documentId}`);

            // Check if we've exceeded max retries
            if (pendingBranchCreations[documentId].attempts >= maxRetries) {
                console.warn(`Max retries (${maxRetries}) reached for document ${documentId}`);
                updateDocumentBranchStatus(documentId, 'failed');
                return false;
            }

            // Increment the retry count
            pendingBranchCreations[documentId].attempts++;
            console.log(`Retry attempt ${pendingBranchCreations[documentId].attempts}/${maxRetries} for document ${documentId}`);
            return true;
        }

        // Mark this document as being fixed
        pendingBranchCreations[documentId] = {
            timestamp: Date.now(),
            attempts: 1
        };

        updateDocumentBranchStatus(documentId, 'fixing');

        // Try multiple approaches to fix the document branch

        // 1. First try to open the document (will cause it to be created)
        const openMsg = {
            type: 'OpenDocument',
            payload: {
                document_id: documentId
            }
        };

        // 2. If the document doesn't exist, create it explicitly
        const createMsg = {
            type: 'CreateDocumentBranch',
            payload: {
                document_id: documentId
            }
        };

        // If socket is open, send the messages
        if (ws.readyState === WebSocket.OPEN) {
            console.log(`Sending OpenDocument to fix branch: ${documentId}`);

            try {
                // First try to open it
                ws.send(JSON.stringify(openMsg));

                // Then after a short delay, try creating it explicitly
                setTimeout(() => {
                    if (ws.readyState === WebSocket.OPEN && pendingBranchCreations[documentId]) {
                        console.log(`Sending CreateDocumentBranch as backup: ${documentId}`);
                        ws.send(JSON.stringify(createMsg));
                    }
                }, 500);

                return true;
            } catch (e) {
                console.error('Failed to send fix branch messages:', e);
                updateDocumentBranchStatus(documentId, 'failed');
                return false;
            }
        } else {
            console.warn('WebSocket not open, cannot fix document branch');
            updateDocumentBranchStatus(documentId, 'failed');
            return false;
        }
    }

    // Function to explicitly create a document branch
    function createDocumentBranch(documentId) {
        if (!documentId) return false;

        // Check if WebSocket is available and connected
        if (!window.websocket || window.websocket.readyState !== WebSocket.OPEN) {
            console.error('Cannot create document branch: WebSocket not connected');
            updateDocumentBranchStatus(documentId, 'failed');
            return false;
        }

        console.log(`Explicitly creating document branch for ${documentId}`);
        updateDocumentBranchStatus(documentId, 'fixing');

        // Create and send the message
        const createMsg = {
            type: 'CreateDocumentBranch',
            payload: {
                document_id: documentId
            }
        };

        try {
            window.websocket.send(JSON.stringify(createMsg));

            // Mark as pending
            pendingBranchCreations[documentId] = {
                timestamp: Date.now(),
                attempts: 1
            };

            return true;
        } catch (e) {
            console.error('Failed to send CreateDocumentBranch message:', e);
            updateDocumentBranchStatus(documentId, 'failed');
            return false;
        }
    }

    // Function to recover document operations after a branch is fixed
    function retryPendingOperations(documentId) {
        if (!documentId) return;

        // Check if we have the document tracker available (from elsewhere in the script)
        if (documentTracker && documentTracker.lastDocOpMessage) {
            // Only retry if this is for the same document
            if (documentTracker.lastDocOpMessage.payload &&
                documentTracker.lastDocOpMessage.payload.operation &&
                documentTracker.lastDocOpMessage.payload.operation.document_id === documentId) {

                console.log(`Retrying last document operation for ${documentId}`);

                // Check if WebSocket is connected
                if (window.websocket && window.websocket.readyState === WebSocket.OPEN) {
                    try {
                        // Set retry flag to avoid loops
                        documentTracker.isRetrying = true;

                        // Send the message
                        window.websocket.send(JSON.stringify(documentTracker.lastDocOpMessage));

                        console.log('Resent last document operation');
                    } catch (e) {
                        console.error('Failed to resend document operation:', e);
                    } finally {
                        // Clear retry flag
                        documentTracker.isRetrying = false;
                    }
                }
            }
        }
    }

    // Update the status of a document branch
    function updateDocumentBranchStatus(documentId, status) {
        if (!documentId) return;

        documentBranchStatus[documentId] = {
            status: status, // 'unknown', 'fixing', 'fixed', 'failed'
            timestamp: Date.now()
        };

        // Update UI if possible
        updateDocumentBranchStatusUI(documentId, status);
    }

    // Update UI to show document branch status
    function updateDocumentBranchStatusUI(documentId, status) {
        // Find status element in the DOM
        const statusElement = document.getElementById('document-branch-status');
        if (!statusElement) return;

        // Update status display
        let statusText = '';
        let statusClass = '';

        switch (status) {
            case 'fixing':
                statusText = `Fixing document branch (${documentId.substring(0, 8)}...)`;
                statusClass = 'text-warning';
                break;
            case 'fixed':
                statusText = `Document branch fixed (${documentId.substring(0, 8)}...)`;
                statusClass = 'text-success';
                // Clear after a few seconds
                setTimeout(() => {
                    if (statusElement && documentBranchStatus[documentId] &&
                        documentBranchStatus[documentId].status === 'fixed') {
                        statusElement.textContent = '';
                        statusElement.className = '';
                    }
                }, 5000);
                break;
            case 'failed':
                statusText = `Failed to fix document branch (${documentId.substring(0, 8)}...)`;
                statusClass = 'text-danger';
                break;
            default:
                statusText = '';
                statusClass = '';
        }

        statusElement.textContent = statusText;
        statusElement.className = statusClass;

        // If this is the current document, also update the document info area
        if (window.currentDocument && window.currentDocument.id === documentId) {
            const docInfoElement = document.getElementById('document-info');
            if (docInfoElement) {
                if (status === 'fixing') {
                    docInfoElement.dataset.branchStatus = 'fixing';
                } else if (status === 'fixed') {
                    docInfoElement.dataset.branchStatus = 'ok';
                } else if (status === 'failed') {
                    docInfoElement.dataset.branchStatus = 'error';
                }
            }
        }
    }

    // Add message handler to monitor for error responses
    const setupMessageHandler = function (ws) {
        // Store the original onmessage handler if it exists
        const originalOnmessage = ws.onmessage;

        // Set new onmessage handler
        ws.onmessage = function (event) {
            // Check if this is an error message
            if (typeof event.data === 'string') {
                try {
                    const message = JSON.parse(event.data);

                    // Check if this is a document branch error
                    if (isDocumentBranchError(message)) {
                        const documentId = extractDocumentIdFromError(message);
                        if (documentId) {
                            console.log(`Detected document branch error for ${documentId}`);

                            // Update status UI
                            updateDocumentBranchStatus(documentId, 'fixing');

                            // Try to fix the document branch
                            if (fixDocumentBranch(ws, documentId)) {
                                // Don't forward the error to the application
                                console.log(`Handling document branch error for ${documentId}`);
                                return;
                            }
                        }
                    }

                    // Check for successful operations that indicate a branch is fixed
                    if (message.type === 'DocumentUpdate' && message.payload && message.payload.document_id) {
                        const documentId = message.payload.document_id;

                        // Check if this document was pending a branch fix
                        if (pendingBranchCreations[documentId]) {
                            console.log(`Document branch appears to be fixed for ${documentId}`);
                            updateDocumentBranchStatus(documentId, 'fixed');

                            // Clean up the pending state
                            delete pendingBranchCreations[documentId];
                        }
                    }

                    // Check for explicit branch creation success
                    if (message.type === 'BranchCreated' && message.payload && message.payload.document_id) {
                        const documentId = message.payload.document_id;
                        console.log(`Document branch explicitly created for ${documentId}`);
                        updateDocumentBranchStatus(documentId, 'fixed');

                        // Clean up the pending state
                        delete pendingBranchCreations[documentId];
                    }
                } catch (e) {
                    // Not JSON or other error, just pass it through
                }
            }

            // Call the original handler if it exists
            if (originalOnmessage) {
                originalOnmessage.call(this, event);
            }
        };
    };

    // Override the WebSocket.send method to ensure protocol compatibility
    WebSocket.prototype.send = function (data) {
        // Only process text messages that are JSON
        if (typeof data === 'string') {
            try {
                // Parse the message
                const message = JSON.parse(data);

                // Set up the message handler if this is a new websocket
                if (this.onmessage && !this._docOpFixInstalled) {
                    setupMessageHandler(this);
                    this._docOpFixInstalled = true;
                }

                // Check if this is a document operation
                if (message.type === 'DocumentOperation' && message.payload && message.payload.operation) {
                    const operation = message.payload.operation;

                    // Store this operation in case we need to retry
                    if (!documentTracker.isRetrying) {
                        documentTracker.lastDocOpMessage = JSON.parse(JSON.stringify(message));
                    }

                    // Check if it's a known document
                    if (operation.document_id) {
                        documentTracker.trackDocument(operation.document_id);
                    }

                    // Transform the operation based on its type
                    if (operation.type === 'Update' || operation.type === 'Replace') {
                        // Convert Update/Replace to Replace operation
                        message.payload.operation = {
                            type: 'Replace',
                            document_id: operation.document_id,
                            range: { start: 0, end: operation.content ? operation.content.length : 10 }, // Replace the entire content
                            content: operation.content || ""
                        };
                        console.log('Document operation fix: Converted Update operation to Replace');
                    } else if (operation.type === 'Insert') {
                        // Fix Insert operation structure if needed
                        if (!operation.position && operation.position !== 0) {
                            operation.position = 0; // Default to beginning if position is missing
                        }

                        // Remove user_id which is not expected in the server's Operation enum
                        if (operation.user_id) {
                            delete operation.user_id;
                        }

                        console.log('Document operation fix: Fixed Insert operation structure');
                    } else if (operation.type === 'Delete') {
                        // Fix Delete operation structure if needed
                        if (!operation.range) {
                            operation.range = { start: 0, end: 0 }; // Default empty range
                        }

                        // Remove user_id which is not expected in the server's Operation enum
                        if (operation.user_id) {
                            delete operation.user_id;
                        }

                        console.log('Document operation fix: Fixed Delete operation structure');
                    }

                    // Replace the data with the fixed message
                    data = JSON.stringify(message);
                }
            } catch (e) {
                // Not JSON or other error, just pass it through
                console.warn('Document operation fix: Error processing message', e);
            }
        }

        // Call the original send method
        return originalSend.call(this, data);
    };

    console.log('Document operation protocol fix loaded');

    // Find any existing WebSocket connections and set up message handlers
    setTimeout(() => {
        if (window.WebSocket && window.WebSocket.prototype) {
            console.log('Document operation fix: Setting up handlers for existing WebSockets');

            // Override the WebSocket constructor to ensure new WebSockets get handlers
            const OriginalWebSocket = window.WebSocket;
            window.WebSocket = function (url, protocols) {
                const ws = new OriginalWebSocket(url, protocols);

                // Wait until the websocket is open to set up our handler
                ws.addEventListener('open', () => {
                    if (!ws._docOpFixInstalled) {
                        setupMessageHandler(ws);
                        ws._docOpFixInstalled = true;
                    }
                });

                return ws;
            };
            window.WebSocket.prototype = OriginalWebSocket.prototype;
            window.WebSocket.CONNECTING = OriginalWebSocket.CONNECTING;
            window.WebSocket.OPEN = OriginalWebSocket.OPEN;
            window.WebSocket.CLOSING = OriginalWebSocket.CLOSING;
            window.WebSocket.CLOSED = OriginalWebSocket.CLOSED;
        }
    }, 500);

    // Make functions available to the window/global scope
    window.createDocumentBranch = createDocumentBranch;
    window.fixDocumentBranch = function (documentId) {
        return fixDocumentBranch(window.websocket, documentId);
    };
    window.retryPendingOperations = retryPendingOperations;
})();
