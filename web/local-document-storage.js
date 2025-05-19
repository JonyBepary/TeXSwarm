/**
 * TeXSwarm - Local Document Storage
 *
 * This script adds local document storage to ensure documents are saved
 * even when the server is unable to persist them properly.
 */

(function () {
    // Keep track of documents and their content
    const localDocuments = {
        // Load saved documents from localStorage
        getAll: function () {
            try {
                return JSON.parse(localStorage.getItem('texswarm_documents') || '{}');
            } catch (e) {
                console.error('Error loading local documents:', e);
                return {};
            }
        },

        // Save documents to localStorage
        saveAll: function (documents) {
            try {
                localStorage.setItem('texswarm_documents', JSON.stringify(documents));
            } catch (e) {
                console.error('Error saving local documents:', e);
            }
        },

        // Get a specific document
        get: function (documentId) {
            const docs = this.getAll();
            return docs[documentId];
        },

        // Save a specific document
        save: function (documentId, document) {
            const docs = this.getAll();
            docs[documentId] = document;
            this.saveAll(docs);

            console.log(`Local document saved: ${documentId}`);
            return document;
        },

        // Delete a document
        delete: function (documentId) {
            const docs = this.getAll();
            if (docs[documentId]) {
                delete docs[documentId];
                this.saveAll(docs);
                console.log(`Local document deleted: ${documentId}`);
                return true;
            }
            return false;
        }
    };

    // Monitor document changes and save them locally
    function setupDocumentTracking() {
        // Find the editor instance
        const findEditor = () => {
            return window.editor || null;
        };

        // Find current document information
        const getCurrentDocumentInfo = () => {
            if (window.currentDocument) {
                return {
                    id: window.currentDocument.id,
                    title: window.currentDocument.title,
                    owner: window.currentDocument.owner,
                    updated_at: new Date().toISOString()
                };
            }
            return null;
        };

        // Save the current document
        const saveCurrentDocument = () => {
            const editor = findEditor();
            const docInfo = getCurrentDocumentInfo();

            if (editor && docInfo) {
                const content = editor.getValue();

                // Save document to local storage
                localDocuments.save(docInfo.id, {
                    ...docInfo,
                    content: content,
                    savedAt: new Date().toISOString()
                });

                return true;
            }

            return false;
        };

        // Set up auto-save
        let autoSaveInterval = null;

        const startAutoSave = () => {
            if (autoSaveInterval) {
                clearInterval(autoSaveInterval);
            }

            // Save every 30 seconds
            autoSaveInterval = setInterval(() => {
                if (saveCurrentDocument()) {
                    console.log('Auto-saved document to local storage');
                }
            }, 30000);
        };

        const stopAutoSave = () => {
            if (autoSaveInterval) {
                clearInterval(autoSaveInterval);
                autoSaveInterval = null;
            }
        };

        // Monitor document changes
        const originalSetCurrentDocument = window.setCurrentDocument || function () { };
        window.setCurrentDocument = function (document) {
            // Call the original function
            originalSetCurrentDocument.apply(this, arguments);

            // Start auto-save for the new document
            if (document) {
                console.log('Starting auto-save for document:', document.id);
                startAutoSave();
            } else {
                stopAutoSave();
            }
        };

        // Add recovery UI
        addRecoveryUI();

        console.log('Local document storage initialized');
    }

    // Add UI for document recovery
    function addRecoveryUI() {
        // Create recovery button
        const recoveryButton = document.createElement('button');
        recoveryButton.className = 'btn btn-outline-warning btn-sm ms-2';
        recoveryButton.innerHTML = '<i class="bi bi-cloud-arrow-down"></i> Recover Local';
        recoveryButton.title = 'Recover locally saved documents';

        // Add button to actions area
        const actionsArea = document.querySelector('.document-actions');
        if (actionsArea) {
            actionsArea.appendChild(recoveryButton);
        } else {
            // Try the user actions area if document actions doesn't exist
            const userSection = document.getElementById('user-section');
            if (userSection) {
                recoveryButton.className = 'btn btn-outline-warning btn-sm mt-2 w-100';
                userSection.appendChild(recoveryButton);
            }
        }

        // Create recovery modal
        const modalHtml = `
            <div class="modal fade" id="recovery-modal" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog modal-lg">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">Recover Local Documents</h5>
                            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                        </div>
                        <div class="modal-body">
                            <div class="table-responsive">
                                <table class="table" id="recovery-docs-table">
                                    <thead>
                                        <tr>
                                            <th>Title</th>
                                            <th>Last Saved</th>
                                            <th>Actions</th>
                                        </tr>
                                    </thead>
                                    <tbody id="recovery-docs-list">
                                        <tr>
                                            <td colspan="3" class="text-center">No documents found</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
                        </div>
                    </div>
                </div>
            </div>
        `;

        // Add modal to body
        const modalContainer = document.createElement('div');
        modalContainer.innerHTML = modalHtml;
        document.body.appendChild(modalContainer);

        // Set up recovery modal
        recoveryButton.addEventListener('click', () => {
            showRecoveryModal();
        });
    }

    // Show the recovery modal with locally saved documents
    function showRecoveryModal() {
        const docs = localDocuments.getAll();
        const docsList = document.getElementById('recovery-docs-list');

        if (!docsList) return;

        // Clear current list
        docsList.innerHTML = '';

        // Check if there are any documents
        if (Object.keys(docs).length === 0) {
            docsList.innerHTML = '<tr><td colspan="3" class="text-center">No documents found</td></tr>';

            // Show the modal
            new bootstrap.Modal(document.getElementById('recovery-modal')).show();
            return;
        }

        // Add each document to the list
        for (const [id, doc] of Object.entries(docs)) {
            const row = document.createElement('tr');

            // Format date
            const savedDate = new Date(doc.savedAt);
            const formattedDate = savedDate.toLocaleString();

            row.innerHTML = `
                <td>${doc.title || 'Untitled'}</td>
                <td>${formattedDate}</td>
                <td>
                    <button class="btn btn-sm btn-primary recover-doc-btn" data-doc-id="${id}">
                        Recover
                    </button>
                    <button class="btn btn-sm btn-outline-danger delete-doc-btn" data-doc-id="${id}">
                        Delete
                    </button>
                </td>
            `;

            docsList.appendChild(row);
        }

        // Add event listeners to buttons
        document.querySelectorAll('.recover-doc-btn').forEach(btn => {
            btn.addEventListener('click', (event) => {
                const docId = event.target.getAttribute('data-doc-id');
                recoverDocument(docId);
            });
        });

        document.querySelectorAll('.delete-doc-btn').forEach(btn => {
            btn.addEventListener('click', (event) => {
                const docId = event.target.getAttribute('data-doc-id');
                deleteLocalDocument(docId);
                event.target.closest('tr').remove();

                // Check if there are any documents left
                if (document.querySelectorAll('#recovery-docs-list tr').length === 0) {
                    docsList.innerHTML = '<tr><td colspan="3" class="text-center">No documents found</td></tr>';
                }
            });
        });

        // Show the modal
        new bootstrap.Modal(document.getElementById('recovery-modal')).show();
    }

    // Recover a document from local storage
    function recoverDocument(docId) {
        const doc = localDocuments.get(docId);
        if (!doc) return;

        console.log('Recovering document:', docId);

        // If there's an editor and setCurrentDocument function
        if (window.editor && typeof window.setCurrentDocument === 'function') {
            // Create a document object compatible with the app
            const documentObj = {
                id: docId,
                title: doc.title || 'Recovered Document',
                owner: doc.owner || 'local-user',
                content: doc.content || '',
                updated_at: doc.savedAt || new Date().toISOString()
            };

            // Set it as the current document
            window.setCurrentDocument(documentObj);

            // Set the editor content
            window.editor.setValue(doc.content || '');

            // Close the modal
            const modal = bootstrap.Modal.getInstance(document.getElementById('recovery-modal'));
            if (modal) modal.hide();
        } else {
            alert('Editor not ready. Please try again later.');
        }
    }

    // Delete a document from local storage
    function deleteLocalDocument(docId) {
        localDocuments.delete(docId);
    }

    // Wait for the DOM to be ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupDocumentTracking);
    } else {
        setupDocumentTracking();
    }
})();
