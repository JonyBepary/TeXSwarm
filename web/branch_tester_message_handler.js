/**
 * TeXSwarm - Message Handler for Document Branch Tester
 *
 * This script adds message handling functionality specifically for the branch tester.
 */

// Setup WebSocket message handler if not already available
(function () {
    // Define a custom message handler for the branch tester
    function setupMessageHandler() {
        if (!window.websocket) {
            console.warn('WebSocket not available, cannot setup message handler');
            return;
        }

        // Check if we already have an onmessage handler
        const originalOnMessage = window.websocket.onmessage;

        // Add our custom handler
        window.websocket.onmessage = function (event) {
            try {
                // Parse the message
                const message = JSON.parse(event.data);
                console.log('Received message:', message.type);

                // Handle specific message types
                if (message.type === 'LoginResponse') {
                    handleLoginResponse(message.payload);
                } else if (message.type === 'Error') {
                    handleErrorMessage(message.payload);
                } else if (message.type === 'DocumentBranchStatus') {
                    handleBranchStatus(message.payload);
                } else if (message.type === 'DocumentCreated') {
                    handleDocumentCreated(message.payload);
                }

                // Call the original handler if it exists
                if (originalOnMessage) {
                    originalOnMessage.call(window.websocket, event);
                }
            } catch (error) {
                console.error('Error processing message:', error);
                // Still call the original handler for non-JSON messages
                if (originalOnMessage) {
                    originalOnMessage.call(window.websocket, event);
                }
            }
        };

        console.log('Branch tester message handler setup complete');
    }

    // Handle login response
    function handleLoginResponse(payload) {
        if (payload && payload.success) {
            console.log('Login successful:', payload);

            // Update UI if needed
            const loginStatus = document.getElementById('login-status');
            if (loginStatus) {
                loginStatus.textContent = `Logged in as ${payload.username || 'User'}`;
                loginStatus.classList.add('logged-in');
            }

            // Hide login button, show logout
            const loginBtn = document.getElementById('login-btn');
            const logoutBtn = document.getElementById('logout-btn');
            if (loginBtn) loginBtn.style.display = 'none';
            if (logoutBtn) logoutBtn.style.display = 'inline-block';
        } else {
            console.error('Login failed:', payload?.error || 'Unknown error');
        }
    }

    // Handle error messages
    function handleErrorMessage(payload) {
        console.error('Server error:', payload?.message || 'Unknown error');

        // Check for document branch errors
        if (payload && payload.message && payload.message.includes('Document branch not found')) {
            console.warn('Document branch error detected');

            // Extract document ID if present
            const match = payload.message.match(/Document branch not found: ([a-zA-Z0-9-]+)/);
            if (match && match[1]) {
                const documentId = match[1];
                console.log('Document ID from error:', documentId);

                // Update branch status
                document.getElementById('branch-status').textContent = 'Error: Branch not found';
                document.getElementById('branch-status').className = 'error';

                // Set as current document if not already set
                if (!document.getElementById('current-doc-id').textContent ||
                    document.getElementById('current-doc-id').textContent === 'None') {
                    document.getElementById('current-doc-id').textContent = documentId;
                }
            }
        }
    }

    // Handle branch status
    function handleBranchStatus(payload) {
        console.log('Branch status update:', payload);
        if (payload && payload.document_id) {
            document.getElementById('current-doc-id').textContent = payload.document_id;
            document.getElementById('branch-status').textContent = payload.status || 'Unknown';

            // Set appropriate class based on status
            if (payload.status === 'Ready') {
                document.getElementById('branch-status').className = 'success';
            } else if (payload.status === 'Error') {
                document.getElementById('branch-status').className = 'error';
            } else if (payload.status === 'Creating') {
                document.getElementById('branch-status').className = 'warning';
            } else {
                document.getElementById('branch-status').className = '';
            }
        }
    }

    // Handle document created
    function handleDocumentCreated(payload) {
        console.log('Document created:', payload);
        if (payload && payload.id) {
            document.getElementById('current-doc-id').textContent = payload.id;
            document.getElementById('branch-status').textContent = 'Created';
            document.getElementById('branch-status').className = 'success';
        }
    }

    // Setup after page load
    window.addEventListener('DOMContentLoaded', function () {
        // We'll wait a bit to ensure WebSocket is initialized
        setTimeout(setupMessageHandler, 1000);
    });

    // Also set up a function to manually call setup
    window.setupBranchTesterMessageHandler = setupMessageHandler;
})();
