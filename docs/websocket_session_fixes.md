# WebSocket Session Management Fixes

This document outlines the fixes made to improve WebSocket session management in the TeXSwarm application.

## Issue

The original implementation had issues with WebSocket session management:

1. When a user refreshed the page or reconnected, the server would return an error message:
   ```
   ERROR p2p_latex_collab::api::websocket: WebSocket error: API error: Session already exists
   ```

2. This happened because the server tracked sessions by WebSocket connection ID, and when a user refreshed the page, a new WebSocket connection was established, but the server still had the old session registered.

## Solution

We implemented the following changes to address these issues:

### Server-Side Fixes

1. Modified the `register_session` method in `src/api/websocket.rs` to handle re-authentication:
   ```rust
   async fn register_session(&self, session_id: &str, user_id: String) -> Result<()> {
       let mut sessions = self.sessions.write().await;

       // Check if this session already exists
       if sessions.contains_key(session_id) {
           // Instead of returning an error, update the existing session if the user_id is different
           if let Some(session) = sessions.get_mut(session_id) {
               if session.user_id != user_id {
                   // Update user ID if it changed
                   session.user_id = user_id;
                   tracing::info!("Updated session {} with new user ID", session_id);
               } else {
                   tracing::info!("Session {} already exists for user {}", session_id, user_id);
               }
           }
           return Ok(());
       }

       // Create the session (existing code)
       ...
   }
   ```

2. Modified the `handle_message` method to provide better feedback for authentication:
   ```rust
   pub async fn handle_message(&self, session_id: &str, message: ApiMessage) -> Result<Option<ApiMessage>> {
       match message {
           ApiMessage::Authentication { user_id, token: _ } => {
               // In a real system, we would validate the token
               self.register_session(session_id, user_id.clone()).await?;

               // Return a positive authentication response
               Ok(Some(ApiMessage::Error {
                   code: "auth_success".to_string(),
                   message: format!("Authentication successful for user {}", user_id),
               }))
           },
   ```

### Client-Side Fixes

1. Enhanced the error handler to recognize authentication success messages:
   ```javascript
   function handleError(payload) {
       // Check if this is actually a success message disguised as an error
       // This happens with our authentication success message
       if (payload.code === "auth_success") {
           console.log('Authentication success:', payload.message);
           // Request document list after successful authentication
           requestDocumentList();
           return;
       }

       showToast(`Error: ${payload.message}`, 'error');
   }
   ```

## Benefits

These changes have the following benefits:

1. **Improved User Experience**: Users no longer see error messages when refreshing the page
2. **More Robust Session Management**: Sessions are properly updated when users reconnect
3. **Better Feedback**: The client now receives clear indication of authentication success
4. **Smoother Reconnection**: The client can immediately request document lists after successful re-authentication

## Future Improvements

For even more robust session management, we could consider:

1. Using unique user identifiers instead of connection IDs for session management
2. Implementing a token-based authentication system with expiration
3. Adding session cleanup for truly inactive sessions
4. Creating a more explicit authentication response type in the protocol
