# WebSocket Protocol Compatibility Fixes

## Problem 1: Unit Variant Messages

During WebSocket communication, the JavaScript client was encountering errors when attempting to send messages to the Rust server. The specific error was:

```
Failed to parse message: invalid type: map, expected unit variant ApiMessage::ListDocuments
```

This error occurred because the client JavaScript code was automatically including an empty payload object (`{}`) for all message types, while the Rust server expected certain message types (specifically `ListDocuments`) to be unit variants without any payload.

In Rust, a unit variant in an enum is defined without associated data:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ApiMessage {
    // Unit variant (no payload)
    ListDocuments,

    // Non-unit variant with payload
    DocumentList {
        documents: Vec<DocumentSummary>,
    },
    // ...
}
```

## Solution

The fix involves intercepting WebSocket messages before they're sent to the server and removing the payload field for message types that shouldn't have one. This is implemented using a JavaScript proxy that overrides the `WebSocket.prototype.send` method.

### Implementation

1. Created a WebSocket protocol fix script (`websocket-fix.js`) that:
   - Keeps a reference to the original WebSocket.send method
   - Defines a list of message types that should not have a payload (`['ListDocuments']`)
   - Overrides the WebSocket.send method to remove payloads from these message types
   - Logs when a fix is applied

2. Updated application pages to include this script after the main app script:
   - Main application (`new-index.html`)
   - Standalone WebSocket test page (`websocket-test.html`)

### Code

```javascript
(function() {
    // Keep a reference to the original WebSocket.send method
    const originalSend = WebSocket.prototype.send;

    // List of message types that should not have a payload (unit variants in Rust enum)
    // According to /src/api/protocol.rs, only ListDocuments is a unit variant
    const noPayloadTypes = ['ListDocuments'];

    // Override the WebSocket.send method to ensure protocol compatibility
    WebSocket.prototype.send = function(data) {
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
```

## Testing

We created a standalone WebSocket test page (`websocket-test.html`) that specifically tests sending the `ListDocuments` message type to verify the fix works correctly.

The tests confirmed:
- The WebSocket fix script properly removes the payload from `ListDocuments` messages
- The server correctly responds to the fixed message format
- Other message types with required payloads (e.g., `Authentication`) continue to work properly

## Problem 2: Document Operation Format

Another WebSocket protocol issue was found with document operations, which were causing these errors:

```
Failed to parse message: unknown variant `user_id`, expected one of `Insert`, `Delete`, `Replace` at line 1 column 61
Error: CRDT error: Document branch not found: 965ac965-a422-43ff-9862-49aaf0cbff4c
```

### Problem Description

The client JavaScript code was sending document operations with an incorrect format:
1. Including a `user_id` field in the operation payload that the server wasn't expecting
2. Using a different operation type schema (`Update` instead of `Replace`, etc.)
3. Not providing the correct structure for operation types (missing `range` for Replace operations)
4. Attempting to operate on documents that don't exist yet

### Solution

We created an advanced fix script (`document-operation-fix.js`) to address these issues:

1. **Message Format Fixing:**
   - Removing unexpected fields like `user_id` from operations
   - Converting `Update` operations to `Replace` operations with proper ranges
   - Ensuring all required fields are present for each operation type

2. **Error Handling and Recovery:**
   - Tracking document IDs that are known to exist
   - Monitoring WebSocket responses for error messages
   - Automatically creating documents when "Document branch not found" errors occur
   - Retrying the original operation after document creation

### Code Examples

#### Format Fixing
```javascript
// Transform document operations to the correct format
if (message.type === 'DocumentOperation' && message.payload && message.payload.operation) {
    const operation = message.payload.operation;

    // Transform the operation based on its type
    if (operation.type === 'Update' || operation.type === 'Replace') {
        // Convert to Replace operation with proper range
        message.payload.operation = {
            type: 'Replace',
            document_id: operation.document_id,
            range: { start: 0, end: operation.content.length },
            content: operation.content
        };
    } else if (operation.type === 'Insert' || operation.type === 'Delete') {
        // Remove user_id which is not expected in the server's Operation enum
        if (operation.user_id) {
            delete operation.user_id;
        }
    }
}
```

#### Error Recovery
```javascript
// Monitor WebSocket responses for errors
ws.onmessage = function(event) {
    try {
        const response = JSON.parse(event.data);

        // Check for error responses
        if (response.type === 'Error' && response.payload) {
            const errorMessage = response.payload.message || '';

            // Check for document branch not found error
            if (errorMessage.includes('Document branch not found') && lastDocOpMessage) {
                const docId = lastDocOpMessage.payload?.operation?.document_id;

                if (docId) {
                    console.log('Document not found, creating document:', docId);

                    // Create the document and retry the operation
                    // [Code to create document and retry operation]
                }
            }
        }
    } catch (e) {
        console.warn('Error processing response', e);
    }
};
```

## Future Improvements

For a more permanent solution, consider one of these approaches:

1. Update the client application code to match the server's expected message formats
2. Modify the server to be more lenient and accept client message variations
3. Generate client-side code from the server API definitions to ensure protocol compatibility
4. Create a shared protocol definition that both client and server import/use

## Related Files

- `/web/websocket-fix.js`: Fix for unit variant messages
- `/web/document-operation-fix.js`: Fix for document operation format
- `/web/new-index.html`: Main application including the fixes
- `/web/websocket-test.html`: Test page for the WebSocket fixes
- `/src/api/protocol.rs`: Server-side API protocol definition
