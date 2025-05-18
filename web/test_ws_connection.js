#!/usr/bin/env node

// Simple WebSocket connection test in TypeScript style
const WebSocket = require('ws');

// Log setup function to make it pretty
function log(msg, type = 'info') {
    const timestamp = new Date().toISOString();
    const colors = {
        info: '\x1b[36m%s\x1b[0m', // cyan
        success: '\x1b[32m%s\x1b[0m', // green
        error: '\x1b[31m%s\x1b[0m', // red
        warn: '\x1b[33m%s\x1b[0m' // yellow
    };

    console.log(colors[type], `[${timestamp}] ${msg}`);
}

// Default configuration
const HOST = process.argv[2] || 'localhost';
const PORT = process.argv[3] || 8081;

const url = `ws://${HOST}:${PORT}/ws`;
log(`Testing WebSocket connection to ${url}...`);

// Create WebSocket connection
const ws = new WebSocket(url);

// Connection opened
ws.on('open', () => {
    log('WebSocket connection established successfully!', 'success');

    // This confirms that we can connect to the WebSocket server
    // In a real client, we would send proper authentication after connection
    log('Connection test successful - the WebSocket server is accessible', 'success');

    // After a brief delay, close the connection
    setTimeout(() => {
        log('Test completed successfully, closing connection', 'success');
        ws.close();
        process.exit(0);
    }, 1000);
});

// Handle errors
ws.on('error', (error) => {
    log(`WebSocket connection error: ${error.message}`, 'error');
    process.exit(1);
});

// Handle messages
ws.on('message', (data) => {
    try {
        const message = JSON.parse(data);
        log(`Received message: ${JSON.stringify(message, null, 2)}`, 'info');
    } catch (e) {
        log(`Received non-JSON message: ${data.toString()}`, 'warn');
    }
});

// Handle close
ws.on('close', (code, reason) => {
    log(`Connection closed: Code ${code}, Reason: ${reason || 'None'}`, 'warn');
});

// Set a timeout in case nothing happens
setTimeout(() => {
    if (ws.readyState !== WebSocket.OPEN) {
        log('Connection timed out after 5 seconds', 'error');
        process.exit(1);
    }
}, 5000);
