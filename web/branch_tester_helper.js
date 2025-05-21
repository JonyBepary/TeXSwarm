/**
 * TeXSwarm - Document Branch Tester Helper
 *
 * This script provides helper functions for the branch tester interface.
 */

// Helper function to check if app.js functions are initialized
function isAppInitialized() {
    return typeof window.connectWebSocket === 'function' &&
        typeof window.login === 'function';
}

// Initialize WebSocket and API connection
async function initializeConnection() {
    console.log('Initializing connection...');

    // Check if we need to load configurations
    if (!window.API_URL || !window.WS_URL) {
        console.log('Loading configuration...');
        await loadConfig();
    }

    // Connect to WebSocket
    if (typeof window.connectWebSocket === 'function') {
        await window.connectWebSocket();
    } else {
        console.warn('connectWebSocket function not available, using fallback');

        // Define fallback WebSocket connection
        if (!window.websocket || window.websocket.readyState !== WebSocket.OPEN) {
            try {
                // Get WS URL from config
                const wsUrl = window.WS_URL || 'ws://localhost:8091';
                console.log(`Connecting to WebSocket at ${wsUrl}`);

                window.websocket = new WebSocket(wsUrl);

                window.websocket.onopen = function () {
                    console.log('WebSocket connected');
                    document.getElementById('ws-status').textContent = 'Connected';
                    document.getElementById('ws-status').className = 'success';
                };

                window.websocket.onclose = function () {
                    console.log('WebSocket disconnected');
                    document.getElementById('ws-status').textContent = 'Disconnected';
                    document.getElementById('ws-status').className = 'error';
                };

                window.websocket.onerror = function (error) {
                    console.error('WebSocket error:', error);
                    document.getElementById('ws-status').textContent = 'Error';
                    document.getElementById('ws-status').className = 'error';
                };

                // Wait for connection
                await new Promise((resolve, reject) => {
                    const timeout = setTimeout(() => {
                        reject(new Error('WebSocket connection timeout'));
                    }, 5000);

                    window.websocket.onopen = function () {
                        clearTimeout(timeout);
                        resolve();
                    };
                });
            } catch (error) {
                console.error('Failed to connect to WebSocket:', error);
                throw error;
            }
        }
    }

    console.log('Connection initialized');
    return true;
}

// Load configuration
async function loadConfig() {
    try {
        const response = await fetch('/config.json');
        if (!response.ok) {
            throw new Error(`Failed to load config: ${response.status} ${response.statusText}`);
        }

        const config = await response.json();

        // Set global variables
        window.API_URL = `http://${config.document_host}:${config.document_port}`;
        window.WS_URL = `ws://${config.document_host}:${config.document_port}`;

        console.log(`Config loaded: API_URL=${window.API_URL}, WS_URL=${window.WS_URL}`);
        return config;
    } catch (error) {
        console.error('Failed to load config:', error);
        // Set defaults
        window.API_URL = 'http://localhost:8090';
        window.WS_URL = 'ws://localhost:8091';
        console.log('Using default config');
        return null;
    }
}

// Branch tester specific login function
async function branchTesterLogin(username) {
    console.log(`Attempting to login as ${username}...`);

    if (!username) {
        console.error('Username is required');
        throw new Error('Username is required');
    }

    try {
        // First ensure connection is initialized
        await initializeConnection();

        // Use app.js login if available
        if (typeof window.login === 'function') {
            console.log('Using app.js login function');
            return await window.login(username);
        }

        // Fallback login implementation
        console.log('Using fallback login implementation');

        // Create a unique ID for this user
        const userId = 'user-' + Math.random().toString(36).substr(2, 9);

        // Send login message via WebSocket
        const message = {
            type: 'Login',
            payload: {
                username: username,
                user_id: userId
            }
        };

        if (window.websocket && window.websocket.readyState === WebSocket.OPEN) {
            window.websocket.send(JSON.stringify(message));

            // For our test, we'll assume success and update the UI
            const user = {
                username: username,
                id: userId
            };

            // Store in localStorage for persistence
            localStorage.setItem('userId', userId);
            localStorage.setItem('username', username);

            console.log(`Logged in successfully as ${username}`);
            return user;
        } else {
            throw new Error('WebSocket not connected');
        }
    } catch (error) {
        console.error('Login failed:', error);
        throw error;
    }
}
