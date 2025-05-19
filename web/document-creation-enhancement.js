/**
 * TeXSwarm - Document Creation Enhancement
 *
 * This script enhances document creation to make it more reliable
 * by automatically retrying document creation if it fails.
 */

(function () {
    // Track pending document creations
    const pendingDocuments = {};

    // Override the create document function
    const originalCreateDocument = window.createDocument || function () { };

    window.createDocument = function (title, callback) {
        // Generate a unique ID for this creation attempt
        const attemptId = Date.now().toString();
        const originalCallback = callback;

        console.log(`Creating document "${title}"...`);

        // Track this document creation
        pendingDocuments[attemptId] = {
            title: title,
            attempts: 0,
            maxAttempts: 3,
            createdAt: new Date()
        };

        // Create a new callback that monitors for success/failure
        const enhancedCallback = function (result, error) {
            if (error) {
                // Document creation failed
                console.error('Document creation failed:', error);

                const pendingDoc = pendingDocuments[attemptId];
                if (pendingDoc && pendingDoc.attempts < pendingDoc.maxAttempts) {
                    // Increment attempt counter
                    pendingDoc.attempts++;

                    console.log(`Retrying document creation (attempt ${pendingDoc.attempts}/${pendingDoc.maxAttempts})...`);

                    // Retry after a delay
                    setTimeout(() => {
                        // Try with websocket if available
                        if (window.websocket && window.websocket.readyState === WebSocket.OPEN) {
                            const message = {
                                type: 'CreateDocument',
                                payload: {
                                    title: title,
                                    repository_url: null
                                }
                            };

                            console.log('Sending CreateDocument message:', message);
                            window.websocket.send(JSON.stringify(message));
                        } else {
                            // Fall back to original method
                            originalCreateDocument.call(window, title, enhancedCallback);
                        }
                    }, 1000);

                    return;
                }
            } else {
                // Document creation succeeded
                console.log('Document created successfully:', result);

                // Clean up
                delete pendingDocuments[attemptId];
            }

            // Call the original callback
            if (originalCallback) {
                originalCallback(result, error);
            }
        };

        // Call the original function with our enhanced callback
        originalCreateDocument.call(window, title, enhancedCallback);
    };

    // Override the websocket message handler to detect document creation results
    const setupMessageHandler = function () {
        // Check if websocket exists
        if (!window.websocket) {
            console.log('Websocket not available yet, waiting...');
            setTimeout(setupMessageHandler, 1000);
            return;
        }

        const originalOnMessage = window.websocket.onmessage;

        if (originalOnMessage && !window.websocket._docCreateEnhanced) {
            window.websocket.onmessage = function (event) {
                try {
                    const message = JSON.parse(event.data);

                    // Check for document creation success/failure
                    if (message.type === 'Error' && message.payload &&
                        message.payload.message && message.payload.message.includes('Document')) {
                        console.log('Document error detected:', message.payload.message);

                        // Check for any pending creations that might match this
                        const pendingIds = Object.keys(pendingDocuments);

                        if (pendingIds.length > 0) {
                            console.log('Found pending document creations, triggering retry...');

                            // Trigger retry for the oldest pending document
                            pendingIds.sort((a, b) => parseInt(a) - parseInt(b));
                            const oldestId = pendingIds[0];

                            const pendingDoc = pendingDocuments[oldestId];
                            if (pendingDoc && pendingDoc.attempts < pendingDoc.maxAttempts) {
                                pendingDoc.attempts++;

                                console.log(`Auto-retrying document creation for "${pendingDoc.title}" (attempt ${pendingDoc.attempts}/${pendingDoc.maxAttempts})...`);

                                setTimeout(() => {
                                    const message = {
                                        type: 'CreateDocument',
                                        payload: {
                                            title: pendingDoc.title,
                                            repository_url: null
                                        }
                                    };

                                    window.websocket.send(JSON.stringify(message));
                                }, 1000);
                            }
                        }
                    } else if (message.type === 'DocumentUpdate' && message.payload && message.payload.document_id) {
                        // Document was updated, which could mean it was successfully created
                        console.log('Document update received, document exists:', message.payload.document_id);

                        // Clean up any pending documents older than 10 seconds
                        const now = new Date();
                        Object.keys(pendingDocuments).forEach(id => {
                            const pendingDoc = pendingDocuments[id];
                            const ageInSeconds = (now - pendingDoc.createdAt) / 1000;

                            if (ageInSeconds > 10) {
                                console.log(`Cleaning up stale pending document "${pendingDoc.title}"`);
                                delete pendingDocuments[id];
                            }
                        });
                    }
                } catch (e) {
                    // Not JSON or other error
                }

                // Call the original handler
                if (originalOnMessage) {
                    originalOnMessage.call(this, event);
                }
            };

            // Mark as enhanced
            window.websocket._docCreateEnhanced = true;
            console.log('Document creation enhancement installed on websocket');
        }
    };

    // Set up the message handler
    setupMessageHandler();

    // Also set up when the websocket is reconnected
    const originalConnectWebSocket = window.connectWebSocket || function () { };

    window.connectWebSocket = function () {
        // Call the original function
        originalConnectWebSocket.apply(this, arguments);

        // Set up our message handler after a delay
        setTimeout(setupMessageHandler, 1000);
    };

    console.log('Document creation enhancement loaded');
})();
