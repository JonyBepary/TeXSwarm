/**
 * TeXSwarm - Collaborative LaTeX Editor
 * WebSocket Protocol Compatibility Fix
 *
 * This script fixes API protocol compatibility issues between client and server.
 *
 * Problem: The client was sending empty payloads for message types that are
 * defined as unit variants in the Rust server API, causing deserialization errors:
 * "Failed to parse message: invalid type: map, expected unit variant ApiMessage::ListDocuments"
 *
 * Solution: This script intercepts WebSocket messages before they're sent and
 * removes the payload field for message types that don't expect one.
 */

// Load this script after new-app.js to patch WebSocket communication

(function () {
    // Keep a reference to the original WebSocket.send method
    const originalSend = WebSocket.prototype.send;

    // List of message types that should not have a payload (unit variants in Rust enum)
    // According to /src/api/protocol.rs, only ListDocuments is a unit variant
    const noPayloadTypes = ['ListDocuments'];

    // Override the WebSocket.send method to ensure protocol compatibility
    WebSocket.prototype.send = function (data) {
        // Only process text messages that are JSON
        if (typeof data === 'string') {
            try {
                // Parse the message
                const message = JSON.parse(data);

                // Check if this is a type that should not have a payload
                if (noPayloadTypes.includes(message.type) && message.hasOwnProperty('payload')) {
                    // Remove the payload
                    delete message.payload;

                    // Replace the data with the fixed message
                    data = JSON.stringify(message);
                    console.log('WebSocket protocol fix: Removed payload from', message.type, 'message');
                }
            } catch (e) {
                // Not JSON or other error, just pass it through
            }
        }

        // Call the original send method
        return originalSend.call(this, data);
    };

    console.log('WebSocket protocol fix loaded');
})();
