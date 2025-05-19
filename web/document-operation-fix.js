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

    // Add message handler to monitor for error responses
    const setupMessageHandler = function (ws) {
        // Store the original onmessage handler if it exists
        const originalOnmessage = ws.onmessage;

        // Set new onmessage handler
        ws.onmessage = function (event) {
            try {
                const response = JSON.parse(event.data);

                // Check for document-related responses
                if (response.type === 'DocumentList' && response.payload && response.payload.documents) {
                    // Track document IDs from the list
                    response.payload.documents.forEach(doc => {
                        if (doc && doc.id) {
                            documentTracker.trackDocument(doc.id);
                        }
                    });
                }
                // Check for document update responses
                else if (response.type === 'DocumentUpdate' && response.payload && response.payload.document_id) {
                    documentTracker.trackDocument(response.payload.document_id);
                }
                // Check for error responses
                else if (response.type === 'Error' && response.payload) {
                    const errorMessage = response.payload.message || '';

                    // Check for document branch not found error
                    if (errorMessage.includes('Document branch not found') && documentTracker.lastDocOpMessage) {
                        const docId = documentTracker.lastDocOpMessage.payload?.operation?.document_id;

                        if (docId && !documentTracker.isRetrying) {
                            console.log('Document operation fix: Document not found, creating document:', docId);

                            // Set retrying flag
                            documentTracker.isRetrying = true;

                            // Try to create the document
                            setTimeout(() => {
                                try {
                                    // Create a document creation message
                                    const createDocMsg = {
                                        type: 'CreateDocument',
                                        payload: {
                                            title: `Auto-created document ${docId.slice(0, 8)}`,
                                            repository_url: null
                                        }
                                    };

                                    // Send the document creation message
                                    ws.send(JSON.stringify(createDocMsg));

                                    // Then retry the operation after a delay
                                    setTimeout(() => {
                                        console.log('Document operation fix: Retrying original operation');
                                        ws.send(JSON.stringify(documentTracker.lastDocOpMessage));
                                        documentTracker.isRetrying = false;
                                    }, 1000);
                                } catch (e) {
                                    console.error('Document operation fix: Error creating document', e);
                                    documentTracker.isRetrying = false;
                                }
                            }, 500);
                        }
                    }
                }
            } catch (e) {
                console.warn('Document operation fix: Error processing response', e);
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
})();
