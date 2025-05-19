# WebSocket Protocol Permanent Fix

This document describes a permanent solution to the WebSocket protocol compatibility issues between the JavaScript client and Rust server in TeXSwarm.

## Background

The TeXSwarm project encountered two main WebSocket protocol compatibility issues:

1. **Unit Variant Messages**: The JavaScript client was sending empty payload objects (`{}`) for message types that should have no payload in the Rust server (unit variants).

2. **Document Operation Format**: The client was sending document operations with incorrect structure, including:
   - Using `user_id` fields not expected by the server
   - Using `Update` operation type instead of `Replace`
   - Missing proper range structure for operations

## Temporary Fixes

Initially, two patch scripts were created to intercept and fix WebSocket messages:

1. `websocket-fix.js` - Removed payload fields from unit variant message types
2. `document-operation-fix.js` - Fixed document operation formats and added error recovery

While these fixes worked as a temporary solution, they:
- Relied on monkey-patching the WebSocket API
- Added complexity with runtime message transformations
- Required loading additional scripts in the correct order
- Were difficult to maintain as the protocol evolved

## Permanent Solution

The permanent solution is a dedicated WebSocket protocol client (`websocket-protocol.js`) that:

1. **Encapsulates Protocol Knowledge**: Understands the correct message formats and structures
2. **Provides Clean API**: Offers intuitive methods for all message types
3. **Handles Errors**: Includes built-in error recovery
4. **Manages Connection**: Automatically reconnects and maintains heartbeats
5. **Properly Types Messages**: Ensures all messages follow the server's expected schema

### Implementation Details

The `TeXSwarmWebSocketClient` class provides:

#### Message Formatting

```javascript
// Instead of this error-prone approach:
websocket.send(JSON.stringify({
    type: 'DocumentOperation',
    payload: {
        operation: {
            document_id: docId,
            user_id: userId,  // Not expected by server
            content: content,
            type: 'Update'    // Not a valid server operation type
        }
    }
}));

// Use this clean API:
websocketClient.replaceDocumentContent(documentId, content);

// Or with more control:
websocketClient.sendDocumentOperation(documentId, 'Replace', {
    content: content,
    range: { start: 0, end: content.length }
});
```

#### Unit Variant Handling

```javascript
// Instead of this:
websocket.send(JSON.stringify({
    type: 'ListDocuments',
    payload: {}  // Server expects no payload
}));

// Use this:
websocketClient.listDocuments();
```

#### Automatic Error Recovery

The client automatically:
- Detects "Document branch not found" errors
- Creates missing documents when needed
- Maintains a list of known documents
- Reconnects after connection failures
- Monitors heartbeats for connection health

### Usage

First, include the WebSocket protocol client:

```html
<script src="websocket-protocol.js"></script>
```

Then use it in your application:

```javascript
// Create a client
const wsClient = new TeXSwarmWebSocketClient('ws://localhost:8081/ws', {
    onOpen: () => console.log('Connected'),
    onMessage: (message) => handleMessage(message),
    onClose: () => console.log('Disconnected'),
    onError: (error) => console.error('Error:', error),
    onStatusChange: (status) => updateUIStatus(status)
});

// Connect
wsClient.connect();

// Send messages
wsClient.authenticate('user123');
wsClient.listDocuments();
wsClient.createDocument('My Document');
wsClient.openDocument('123e4567-e89b-12d3-a456-426614174000');
wsClient.replaceDocumentContent('123e4567-e89b-12d3-a456-426614174000', 'Hello World');
```

## Migration Guide

To migrate from the temporary fix scripts to the permanent solution:

1. Include `websocket-protocol.js` in your HTML:
   ```html
   <script src="websocket-protocol.js"></script>
   ```

2. Replace direct WebSocket usage with the protocol client:
   ```javascript
   // Replace:
   let websocket = new WebSocket(WS_URL);

   // With:
   let wsClient = new TeXSwarmWebSocketClient(WS_URL, {
       onOpen: handleWebSocketOpen,
       onMessage: handleWebSocketMessage,
       onClose: handleWebSocketClose,
       onError: handleWebSocketError,
       onStatusChange: updateConnectionStatus
   });
   ```

3. Replace direct message sending:
   ```javascript
   // Replace:
   websocket.send(JSON.stringify({ type: 'ListDocuments' }));

   // With:
   wsClient.listDocuments();
   ```

4. Remove the temporary fix scripts once migration is complete:
   ```html
   <!-- Remove these lines -->
   <script src="websocket-fix.js"></script>
   <script src="document-operation-fix.js"></script>
   ```

## Future Improvements

For even better protocol compatibility:

1. **Code Generation**: Generate client code from the Rust protocol definitions
2. **Shared Schema**: Use a common schema format (like JSON Schema) for both client and server
3. **Type Checking**: Add TypeScript definitions for compile-time type checking
4. **Protocol Versioning**: Add protocol version negotiation for backward compatibility
