import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

export class ExtensionError extends Error {
    constructor(message: string, public readonly cause?: Error) {
        super(message);
        this.name = 'ExtensionError';
    }
}

export async function findCargoToml(startPath?: string): Promise<string | undefined> {
    if (!startPath && !vscode.workspace.workspaceFolders?.length) {
        return undefined;
    }

    const searchPath = startPath || vscode.workspace.workspaceFolders![0].uri.fsPath;
    const cargoTomlPath = path.join(searchPath, 'Cargo.toml');

    try {
        await fs.access(cargoTomlPath);
        return cargoTomlPath;
    } catch {
        return undefined;
    }
}

export function formatVersion(version: string): string {
    // Strip any leading 'v' or '^'
    return version.replace(/^[v^]/, '');
}

export function compareVersions(a: string, b: string): number {
    const normA = formatVersion(a);
    const normB = formatVersion(b);
    
    const partsA = normA.split('.').map(Number);
    const partsB = normB.split('.').map(Number);
    
    for (let i = 0; i < Math.max(partsA.length, partsB.length); i++) {
        const partA = partsA[i] || 0;
        const partB = partsB[i] || 0;
        
        if (partA !== partB) {
            return partA - partB;
        }
    }
    
    return 0;
}

export function handleError(error: unknown, message: string): void {
    console.error(error);
    
    if (error instanceof ExtensionError) {
        vscode.window.showErrorMessage(`${message}: ${error.message}`);
        if (error.cause) {
            console.error('Caused by:', error.cause);
        }
    } else if (error instanceof Error) {
        vscode.window.showErrorMessage(`${message}: ${error.message}`);
    } else {
        vscode.window.showErrorMessage(`${message}: An unknown error occurred`);
    }
}

export async function confirmAction(message: string): Promise<boolean> {
    const result = await vscode.window.showWarningMessage(
        message,
        { modal: true },
        'Yes',
        'No'
    );
    return result === 'Yes';
}

export function parseCargoToml(content: string): { [key: string]: any } {
    // Very basic TOML parser for Cargo.toml files
    const result: { [key: string]: any } = {};
    let currentSection: { [key: string]: any } = result;
    let currentSectionName = '';

    const lines = content.split('\n');
    for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) {
            continue;
        }

        // Section header
        const sectionMatch = trimmed.match(/^\[(.*)\]$/);
        if (sectionMatch) {
            currentSectionName = sectionMatch[1];
            const sections = currentSectionName.split('.');
            currentSection = result;
            for (const section of sections) {
                currentSection[section] = currentSection[section] || {};
                currentSection = currentSection[section];
            }
            continue;
        }

        // Key-value pair
        const kvMatch = trimmed.match(/^"?([^"=]+)"?\s*=\s*(.+)$/);
        if (kvMatch) {
            const [, key, value] = kvMatch;
            currentSection[key.trim()] = parseTomlValue(value.trim());
        }
    }

    return result;
}

function parseTomlValue(value: string): any {
    // Remove quotes if present
    if (value.startsWith('"') && value.endsWith('"')) {
        return value.slice(1, -1);
    }

    // Array
    if (value.startsWith('[') && value.endsWith(']')) {
        return value
            .slice(1, -1)
            .split(',')
            .map(v => parseTomlValue(v.trim()))
            .filter(v => v !== '');
    }

    // Table
    if (value.startsWith('{') && value.endsWith('}')) {
        const result: { [key: string]: any } = {};
        const entries = value
            .slice(1, -1)
            .split(',')
            .map(entry => entry.trim())
            .filter(entry => entry !== '');

        for (const entry of entries) {
            const [key, val] = entry.split('=').map(part => part.trim());
            result[key] = parseTomlValue(val);
        }
        return result;
    }

    // Number
    if (!isNaN(Number(value))) {
        return Number(value);
    }

    // Boolean
    if (value === 'true') return true;
    if (value === 'false') return false;

    return value;
}

export function formatDuration(ms: number): string {
    if (ms < 1000) {
        return `${ms}ms`;
    }
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) {
        return `${seconds}s`;
    }
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
}