/**
 * TeXSwarm - Collaborative LaTeX Editor
 * WebSocket Protocol Module
 *
 * This module provides a clean interface for WebSocket communication with the TeXSwarm server,
 * ensuring protocol compatibility and proper message formats.
 */

class TeXSwarmWebSocketClient {
    /**
     * Create a new WebSocket client
     * @param {string} url - WebSocket server URL
     * @param {Object} options - Configuration options
     * @param {function} options.onOpen - Callback when connection opens
     * @param {function} options.onMessage - Callback when message is received
     * @param {function} options.onClose - Callback when connection closes
     * @param {function} options.onError - Callback when error occurs
     * @param {function} options.onStatusChange - Callback when connection status changes
     */
    constructor(url, options = {}) {
        this.url = url;
        this.options = options;
        this.socket = null;
        this.connected = false;
        this.reconnectTimeout = null;
        this.reconnectInterval = 5000; // 5 seconds
        this.maxReconnectAttempts = 10;
        this.reconnectAttempts = 0;
        this.heartbeatInterval = null;
        this.lastHeartbeat = null;
        this.heartbeatTimeout = null;

        // Known document IDs for tracking
        this.knownDocuments = new Set();

        // Unit variant message types (no payload)
        this.noPayloadTypes = ['ListDocuments'];
    }

    /**
     * Connect to the WebSocket server
     */
    connect() {
        if (this.socket && (this.socket.readyState === WebSocket.CONNECTING ||
            this.socket.readyState === WebSocket.OPEN)) {
            return;
        }

        this._setStatus('connecting');

        try {
            this.socket = new WebSocket(this.url);

            this.socket.onopen = (event) => {
                this.connected = true;
                this.reconnectAttempts = 0;
                this._setStatus('connected');

                // Start heartbeat
                this._startHeartbeat();

                if (this.options.onOpen) {
                    this.options.onOpen(event);
                }
            };

            this.socket.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);

                    // Process server response for document tracking
                    this._processServerResponse(message);

                    if (this.options.onMessage) {
                        this.options.onMessage(message, event);
                    }
                } catch (error) {
                    console.warn('Error parsing WebSocket message:', error);

                    if (this.options.onError) {
                        this.options.onError(error);
                    }
                }
            };

            this.socket.onclose = (event) => {
                this.connected = false;
                this._setStatus('disconnected');

                // Stop heartbeat
                this._stopHeartbeat();

                if (this.options.onClose) {
                    this.options.onClose(event);
                }

                // Attempt to reconnect
                this._scheduleReconnect();
            };

            this.socket.onerror = (event) => {
                this._setStatus('error');

                if (this.options.onError) {
                    this.options.onError(event);
                }
            };
        } catch (error) {
            this._setStatus('error');
            console.error('Error connecting to WebSocket:', error);

            if (this.options.onError) {
                this.options.onError(error);
            }

            // Attempt to reconnect
            this._scheduleReconnect();
        }
    }

    /**
     * Disconnect from the WebSocket server
     */
    disconnect() {
        this._stopHeartbeat();

        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }

        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }

        this.connected = false;
        this._setStatus('disconnected');
    }

    /**
     * Send a message to the server
     * @param {Object|string} message - Message to send
     */
    send(message) {
        if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
            console.warn('Cannot send message, WebSocket not connected');
            return false;
        }

        // Handle different message formats
        if (typeof message === 'string') {
            try {
                // Try to parse as JSON to fix format if needed
                const parsed = JSON.parse(message);
                const fixed = this._fixMessageFormat(parsed);
                this.socket.send(JSON.stringify(fixed));
            } catch (e) {
                // Not JSON, send as is
                this.socket.send(message);
            }
        } else if (typeof message === 'object') {
            // Fix message format
            const fixed = this._fixMessageFormat(message);
            this.socket.send(JSON.stringify(fixed));
        } else {
            // Other types, send as is
            this.socket.send(message);
        }

        return true;
    }

    /**
     * Check if WebSocket is connected
     * @returns {boolean} True if connected
     */
    isConnected() {
        return this.socket && this.socket.readyState === WebSocket.OPEN;
    }

    /**
     * Send an authentication message
     * @param {string} userId - User ID
     * @param {string|null} token - Authentication token (optional)
     */
    authenticate(userId, token = null) {
        return this.send({
            type: 'Authentication',
            payload: {
                user_id: userId,
                token: token
            }
        });
    }

    /**
     * Request list of documents
     */
    listDocuments() {
        return this.send({
            type: 'ListDocuments'
            // No payload for ListDocuments (unit variant)
        });
    }

    /**
     * Create a new document
     * @param {string} title - Document title
     * @param {string|null} repositoryUrl - Repository URL (optional)
     */
    createDocument(title, repositoryUrl = null) {
        return this.send({
            type: 'CreateDocument',
            payload: {
                title: title,
                repository_url: repositoryUrl
            }
        });
    }

    /**
     * Open an existing document
     * @param {string} documentId - Document ID (UUID)
     */
    openDocument(documentId) {
        // Add to known documents
        this.knownDocuments.add(documentId);

        return this.send({
            type: 'OpenDocument',
            payload: {
                document_id: documentId
            }
        });
    }

    /**
     * Send a document operation
     * @param {string} documentId - Document ID (UUID)
     * @param {string} operationType - Operation type ('Insert', 'Delete', 'Replace')
     * @param {Object} options - Operation options
     */
    sendDocumentOperation(documentId, operationType, options = {}) {
        // Add to known documents
        this.knownDocuments.add(documentId);

        let operation;

        switch (operationType) {
            case 'Insert':
                operation = {
                    type: 'Insert',
                    document_id: documentId,
                    position: options.position || 0,
                    content: options.content || ''
                };
                break;

            case 'Delete':
                operation = {
                    type: 'Delete',
                    document_id: documentId,
                    range: options.range || { start: 0, end: 0 }
                };
                break;

            case 'Replace':
            default:
                operation = {
                    type: 'Replace',
                    document_id: documentId,
                    range: options.range || {
                        start: 0,
                        end: options.content ? options.content.length : 0
                    },
                    content: options.content || ''
                };
                break;
        }

        return this.send({
            type: 'DocumentOperation',
            payload: {
                operation: operation
            }
        });
    }

    /**
     * Replace the entire document content
     * @param {string} documentId - Document ID (UUID)
     * @param {string} content - New document content
     */
    replaceDocumentContent(documentId, content) {
        return this.sendDocumentOperation(documentId, 'Replace', {
            content: content,
            range: { start: 0, end: content.length }
        });
    }

    /**
     * Send a presence update
     * @param {string} documentId - Document ID (UUID)
     * @param {Object} presence - Presence information
     */
    sendPresenceUpdate(documentId, presence) {
        return this.send({
            type: 'PresenceUpdate',
            payload: {
                document_id: documentId,
                presence: presence
            }
        });
    }

    /**
     * Handle heartbeat from server
     * @private
     */
    _handleHeartbeat(heartbeat) {
        this.lastHeartbeat = new Date();

        // Reset heartbeat timeout
        if (this.heartbeatTimeout) {
            clearTimeout(this.heartbeatTimeout);
        }

        // Set new timeout for heartbeat
        this.heartbeatTimeout = setTimeout(() => {
            console.warn('No heartbeat received for 30 seconds');
            this._setStatus('timeout');
            this.disconnect();
            this._scheduleReconnect();
        }, 30000); // 30 seconds
    }

    /**
     * Start heartbeat checking
     * @private
     */
    _startHeartbeat() {
        this.lastHeartbeat = new Date();

        // Clear existing intervals
        this._stopHeartbeat();

        // Check heartbeat every 10 seconds
        this.heartbeatInterval = setInterval(() => {
            // If we haven't received a heartbeat in 30 seconds, reconnect
            const now = new Date();
            if (this.lastHeartbeat && (now - this.lastHeartbeat > 30000)) {
                console.warn('No heartbeat received for 30 seconds');
                this._setStatus('timeout');
                this.disconnect();
                this._scheduleReconnect();
            }
        }, 10000); // 10 seconds
    }

    /**
     * Stop heartbeat checking
     * @private
     */
    _stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }

        if (this.heartbeatTimeout) {
            clearTimeout(this.heartbeatTimeout);
            this.heartbeatTimeout = null;
        }
    }

    /**
     * Schedule a reconnection attempt
     * @private
     */
    _scheduleReconnect() {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }

        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;

            this.reconnectTimeout = setTimeout(() => {
                console.log(`Reconnecting (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
                this.connect();
            }, this.reconnectInterval);
        } else {
            console.error('Maximum reconnection attempts reached');
            this._setStatus('failed');
        }
    }

    /**
     * Process server response for document tracking
     * @param {Object} message - Message from server
     * @private
     */
    _processServerResponse(message) {
        // Track document IDs
        if (message.type === 'DocumentList' && message.payload && message.payload.documents) {
            message.payload.documents.forEach(doc => {
                if (doc && doc.id) {
                    this.knownDocuments.add(doc.id);
                }
            });
        } else if (message.type === 'DocumentUpdate' && message.payload && message.payload.document_id) {
            this.knownDocuments.add(message.payload.document_id);
        } else if (message.type === 'Error' && message.payload) {
            const errorMessage = message.payload.message || '';

            // Auto-create missing documents
            if (errorMessage.includes('Document branch not found')) {
                const docId = this._extractDocumentIdFromError(errorMessage);

                if (docId) {
                    console.log(`Document not found, creating document: ${docId}`);
                    setTimeout(() => this.createDocument(`Auto-created document ${docId.slice(0, 8)}`), 500);
                }
            }
        } else if (message.type === 'Heartbeat' && message.payload) {
            this._handleHeartbeat(message.payload);
        }
    }

    /**
     * Extract document ID from error message
     * @param {string} errorMessage - Error message
     * @returns {string|null} Document ID or null
     * @private
     */
    _extractDocumentIdFromError(errorMessage) {
        const match = errorMessage.match(/Document branch not found: ([0-9a-f-]+)/i);
        return match ? match[1] : null;
    }

    /**
     * Fix message format
     * @param {Object} message - Message to fix
     * @returns {Object} Fixed message
     * @private
     */
    _fixMessageFormat(message) {
        // Make a copy to avoid modifying the original
        const fixed = JSON.parse(JSON.stringify(message));

        // Handle unit variants (no payload)
        if (this.noPayloadTypes.includes(fixed.type) && fixed.hasOwnProperty('payload')) {
            delete fixed.payload;
        }

        // Fix document operations
        if (fixed.type === 'DocumentOperation' && fixed.payload && fixed.payload.operation) {
            const operation = fixed.payload.operation;

            // Remove user_id which is not expected
            if (operation.user_id) {
                delete operation.user_id;
            }

            // Transform Update to Replace
            if (operation.type === 'Update') {
                fixed.payload.operation = {
                    type: 'Replace',
                    document_id: operation.document_id,
                    range: { start: 0, end: operation.content ? operation.content.length : 0 },
                    content: operation.content || ''
                };
            }
        }

        return fixed;
    }

    /**
     * Set connection status and trigger callback
     * @param {string} status - Connection status
     * @private
     */
    _setStatus(status) {
        if (this.options.onStatusChange) {
            this.options.onStatusChange(status);
        }
    }
}

// Export the class
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TeXSwarmWebSocketClient;
} else {
    window.TeXSwarmWebSocketClient = TeXSwarmWebSocketClient;
}
