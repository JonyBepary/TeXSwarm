#!/usr/bin/env node

// Simple WebSocket connection test
const WebSocket = require('ws');

// Default configuration
const HOST = process.argv[2] || 'localhost';
const PORT = process.argv[3] || 8081;

const url = `ws://${HOST}:${PORT}/ws`;
console.log(`Testing WebSocket connection to ${url}...`);

// Create WebSocket connection
const ws = new WebSocket(url);

// Connection opened
ws.on('open', () => {
    console.log('✅ SUCCESS: WebSocket connection established!');

    // Send a simple authentication message
    const authMessage = {
        type: 'Authentication',
        user_id: 'test_user',
        token: null
    };

    console.log('Sending authentication message...');
    ws.send(JSON.stringify(authMessage));
});

// Handle errors
ws.on('error', (error) => {
    console.error(`❌ ERROR: ${error.message}`);
    process.exit(1);
});

// Handle messages
ws.on('message', (data) => {
    try {
        const message = JSON.parse(data);
        console.log('Received message:', message);
    } catch (e) {
        console.log('Received non-JSON message:', data.toString());
    }

    // Close the connection after a brief delay
    setTimeout(() => {
        console.log('Test completed, closing connection');
        ws.close();
        process.exit(0);
    }, 1000);
});

// Handle close
ws.on('close', (code, reason) => {
    console.log(`Connection closed: Code ${code}, Reason: ${reason || 'None'}`);
});

// Set a timeout in case nothing happens
setTimeout(() => {
    if (ws.readyState !== WebSocket.OPEN) {
        console.error('❌ ERROR: Connection timed out');
        process.exit(1);
    }
}, 5000);
