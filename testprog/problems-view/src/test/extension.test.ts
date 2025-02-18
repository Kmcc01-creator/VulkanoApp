import * as assert from 'assert';
import { describe, it } from 'mocha';
// Import any other necessary VS Code testing utilities
import * as vscode from 'vscode';

describe('Extension Test Suite', () => {
    // Runs before all tests
    before(() => {
        vscode.window.showInformationMessage('Start all tests.');
    });

    it('Sample test', () => {
        assert.strictEqual(-1, [1, 2, 3].indexOf(5));
        assert.strictEqual(2, [1, 2, 3].indexOf(3));
    });
});