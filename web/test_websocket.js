// A simple WebSocket test client for the P2P LaTeX collaboration system
// Run this with Node.js: node test_websocket.js

const WebSocket = require('ws');

// Connect to the WebSocket server
const ws = new WebSocket('ws://localhost:8081');

// Document ID from our previous tests
const documentId = 'ca67a9c2-7231-4eae-ba7f-7f76ddbcdd45';
const userId = 'test_client_user';

// Handle WebSocket events
ws.on('open', () => {
    console.log('Connected to WebSocket server');

    // Send an authentication message
    sendMessage({
        type: 'auth',
        user_id: userId,
        session_token: 'test-token'
    });

    // After a short delay, join the document
    setTimeout(() => {
        sendMessage({
            type: 'join_document',
            document_id: documentId
        });
    }, 1000);

    // After another delay, send an operation
    setTimeout(() => {
        sendMessage({
            type: 'operation',
            document_id: documentId,
            operation: {
                type: 'insert',
                position: 10,
                content: 'WebSocket Test'
            }
        });
    }, 2000);
});

ws.on('message', (data) => {
    const message = JSON.parse(data);
    console.log('Received message:', message);

    // Handle specific message types
    if (message.type === 'error') {
        console.error('Server error:', message.message);
    }
});

ws.on('error', (error) => {
    console.error('WebSocket error:', error);
});

ws.on('close', () => {
    console.log('Disconnected from WebSocket server');
});

// Helper function to send messages
function sendMessage(message) {
    console.log('Sending message:', message);
    ws.send(JSON.stringify(message));
}

// Close connection after 10 seconds
setTimeout(() => {
    console.log('Test complete, closing connection');
    ws.close();
}, 10000);
