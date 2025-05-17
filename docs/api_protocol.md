# API Protocol Documentation

This document describes the API protocol for the P2P LaTeX Collaboration Tool.

## Overview

The API protocol defines the messages that can be exchanged between clients and the server over WebSockets, as well as the REST API endpoints for HTTP communication.

## WebSocket Protocol

### Message Format

All WebSocket messages follow this JSON format:

```json
{
  "type": "MessageType",
  "payload": {
    // Message-specific fields
  }
}
```

### Message Types

#### Authentication

Used to authenticate a client with the server.

```json
{
  "type": "Authentication",
  "payload": {
    "user_id": "user-123",
    "token": "optional-auth-token"
  }
}
```

#### DocumentOperation

Used to send document operations from clients to the server.

```json
{
  "type": "DocumentOperation",
  "payload": {
    "operation": {
      "type": "Insert",
      "document_id": "uuid-string",
      "position": 10,
      "content": "Hello, world!"
    }
  }
}
```

Operation types:
- `Insert`: Insert text at a position
- `Delete`: Delete text in a range
- `Replace`: Replace text in a range with new content

#### DocumentUpdate

Sent from the server to clients when a document is updated.

```json
{
  "type": "DocumentUpdate",
  "payload": {
    "document_id": "uuid-string",
    "content": "Full document content",
    "version": "latest"
  }
}
```

#### CreateDocument

Used to create a new document.

```json
{
  "type": "CreateDocument",
  "payload": {
    "title": "My LaTeX Document",
    "repository_url": "https://github.com/user/repo.git"
  }
}
```

#### OpenDocument

Used to open an existing document.

```json
{
  "type": "OpenDocument",
  "payload": {
    "document_id": "uuid-string"
  }
}
```

#### PresenceUpdate

Used to broadcast user presence information.

```json
{
  "type": "PresenceUpdate",
  "payload": {
    "document_id": "uuid-string",
    "presence": {
      "user_id": "user-123",
      "cursor_position": 42,
      "selection_range": [42, 50],
      "last_activity": "2023-08-15T12:34:56Z"
    }
  }
}
```

#### ListDocuments

Used to request a list of available documents.

```json
{
  "type": "ListDocuments"
}
```

#### DocumentList

Sent from the server in response to a ListDocuments request.

```json
{
  "type": "DocumentList",
  "payload": {
    "documents": [
      {
        "id": "uuid-string-1",
        "title": "Document 1",
        "owner": "user-123",
        "collaborators": ["user-456", "user-789"],
        "repository_url": "https://github.com/user/repo1.git",
        "created_at": "2023-08-15T10:00:00Z",
        "updated_at": "2023-08-15T11:30:00Z"
      },
      {
        "id": "uuid-string-2",
        "title": "Document 2",
        "owner": "user-456",
        "collaborators": ["user-123"],
        "repository_url": null,
        "created_at": "2023-08-14T15:00:00Z",
        "updated_at": "2023-08-15T09:45:00Z"
      }
    ]
  }
}
```

#### Error

Sent from the server when an error occurs.

```json
{
  "type": "Error",
  "payload": {
    "code": "AUTH_ERROR",
    "message": "Authentication failed"
  }
}
```

## HTTP API

### Endpoints

#### Create Document

- **URL**: `/documents`
- **Method**: `POST`
- **Request Body**:
  ```json
  {
    "title": "My Document",
    "owner": "user-123"
  }
  ```
- **Response**:
  ```json
  {
    "document_id": "uuid-string"
  }
  ```

#### List Documents

- **URL**: `/documents`
- **Method**: `GET`
- **Response**:
  ```json
  {
    "documents": [
      {
        "id": "uuid-string-1",
        "title": "Document 1",
        "owner": "user-123",
        "collaborators": ["user-456", "user-789"],
        "repository_url": "https://github.com/user/repo1.git",
        "created_at": "2023-08-15T10:00:00Z",
        "updated_at": "2023-08-15T11:30:00Z"
      },
      // ...more documents
    ]
  }
  ```

#### Get Document

- **URL**: `/documents/{id}`
- **Method**: `GET`
- **Response**:
  ```json
  {
    "id": "uuid-string-1",
    "title": "Document 1",
    "owner": "user-123",
    "collaborators": ["user-456", "user-789"],
    "repository_url": "https://github.com/user/repo1.git",
    "created_at": "2023-08-15T10:00:00Z",
    "updated_at": "2023-08-15T11:30:00Z"
  }
  ```

#### Insert Operation

- **URL**: `/documents/{id}/insert`
- **Method**: `POST`
- **Request Body**:
  ```json
  {
    "user_id": "user-123",
    "position": 10,
    "content": "Hello, world!"
  }
  ```
- **Response**:
  ```json
  {
    "success": true
  }
  ```

#### Delete Operation

- **URL**: `/documents/{id}/delete`
- **Method**: `POST`
- **Request Body**:
  ```json
  {
    "user_id": "user-123",
    "start": 10,
    "end": 20
  }
  ```
- **Response**:
  ```json
  {
    "success": true
  }
  ```

#### Git Synchronization

- **URL**: `/documents/{id}/sync`
- **Method**: `POST`
- **Response**:
  ```json
  {
    "success": true
  }
  ```
