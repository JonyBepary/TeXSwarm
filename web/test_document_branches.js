/**
 * TeXSwarm - Document Branch Test Script
 *
 * This script can be executed in the browser console to test the document branch
 * handling functionality of the TeXSwarm editor.
 *
 * How to use:
 * 1. Open the TeXSwarm editor (new-index.html)
 * 2. Login to your account
 * 3. Open the browser console (F12)
 * 4. Copy and paste this entire script into the console
 * 5. Execute the commands shown at the bottom
 */

// Test document branch functionality
const DocumentBranchTester = {
    // Create test document for branch testing
    createTestDocument: async function (name = null) {
        const testName = name || `Branch Test Doc ${new Date().toISOString().slice(0, 16).replace('T', ' ')}`;
        console.log(`Creating test document "${testName}"...`);

        return new Promise((resolve, reject) => {
            // Check if we have necessary functions
            if (!window.createDocument) {
                reject(new Error('createDocument function not available'));
                return;
            }

            // Create document and handle response
            try {
                window.createDocument(testName, (doc, error) => {
                    if (error) {
                        console.error('Failed to create document:', error);
                        reject(error);
                    } else {
                        console.log('Document created:', doc);
                        resolve(doc);
                    }
                });
            } catch (e) {
                reject(e);
            }
        });
    },

    // Simulate document branch error
    simulateDocumentBranchError: async function (documentId) {
        if (!documentId) {
            console.error('No document ID provided');
            return false;
        }

        console.log(`Simulating document branch error for ${documentId}`);

        // Create an error message
        const errorMessage = {
            type: 'Error',
            payload: {
                message: `Document branch not found: ${documentId}`
            }
        };

        // Dispatch a message event on the websocket
        if (window.websocket && window.websocket.onmessage) {
            try {
                // Create a fake MessageEvent
                const event = new MessageEvent('message', {
                    data: JSON.stringify(errorMessage)
                });

                // Call the handler
                window.websocket.onmessage(event);
                console.log('Simulated error message dispatched');
                return true;
            } catch (e) {
                console.error('Failed to dispatch simulated error:', e);
                return false;
            }
        } else {
            console.error('WebSocket or message handler not available');
            return false;
        }
    },

    // Test the fix for a document branch
    testDocumentBranchFix: async function (documentId) {
        if (!documentId) {
            console.error('No document ID provided');
            return false;
        }

        console.log(`Testing document branch fix for ${documentId}`);

        // Check if we have the fix function available
        if (window.fixDocumentBranch) {
            try {
                window.fixDocumentBranch(documentId);
                console.log('Fix initiated');
                return true;
            } catch (e) {
                console.error('Failed to initiate fix:', e);
                return false;
            }
        } else {
            console.error('fixDocumentBranch function not available');
            return false;
        }
    },

    // Test the entire document branch workflow
    testCompleteBranchWorkflow: async function () {
        try {
            console.log('Starting complete document branch workflow test...');

            // 1. Create a test document
            console.log('Step 1: Creating test document...');
            const doc = await this.createTestDocument();
            if (!doc || !doc.id) {
                throw new Error('Failed to create test document');
            }
            console.log(`Document created with ID: ${doc.id}`);

            // 2. Wait a moment
            await new Promise(resolve => setTimeout(resolve, 1000));

            // 3. Simulate a branch error
            console.log('Step 2: Simulating branch error...');
            const errorSimulated = await this.simulateDocumentBranchError(doc.id);
            if (!errorSimulated) {
                throw new Error('Failed to simulate branch error');
            }

            // 4. Wait a moment for error handlers to run
            await new Promise(resolve => setTimeout(resolve, 2000));

            // 5. Test fix function
            console.log('Step 3: Testing fix function...');
            const fixInitiated = await this.testDocumentBranchFix(doc.id);
            if (!fixInitiated) {
                throw new Error('Failed to initiate fix');
            }

            console.log('Test completed successfully!');
            return true;
        } catch (e) {
            console.error('Test workflow failed:', e);
            return false;
        }
    }
};

// Commands that can be executed in the console:
console.log(`
Document Branch Tester loaded!
Use the following commands to test document branch functionality:

1. Create test document:
   DocumentBranchTester.createTestDocument()

2. Simulate branch error (replace with your document id):
   DocumentBranchTester.simulateDocumentBranchError('your-document-id')

3. Test fix function (replace with your document id):
   DocumentBranchTester.testDocumentBranchFix('your-document-id')

4. Run complete workflow test:
   DocumentBranchTester.testCompleteBranchWorkflow()
`);
