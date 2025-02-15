import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface ExecuteResult {
    stdout: string;
    stderr: string;
}

export interface DependencyInfo {
    name: string;
    currentVersion: string;
    latestVersion?: string;
    updateRecommended: boolean;
    vulnerabilities: string[];
}

export interface AnalysisReport {
    dependencies: DependencyInfo[];
    auditOutput: string;
}

export class VersionTrackingError extends Error {
    constructor(message: string, public readonly originalError?: Error) {
        super(message);
        this.name = 'VersionTrackingError';
    }
}

export class VersionTrackingClient {
    private outputChannel: vscode.OutputChannel;
    
    constructor() {
        this.outputChannel = vscode.window.createOutputChannel('Version Tracking');
    }

    private getExecutablePath(): string {
        const config = vscode.workspace.getConfiguration('versionTracking');
        return config.get('executablePath') || 'versiontracking';
    }

    private async execute(command: string): Promise<ExecuteResult> {
        try {
            const executablePath = this.getExecutablePath();
            const { stdout, stderr } = await execAsync(`${executablePath} ${command}`);
            
            if (stderr) {
                this.outputChannel.appendLine(`Warning: ${stderr}`);
            }
            
            return { stdout, stderr };
        } catch (error) {
            throw new VersionTrackingError(
                `Failed to execute command: ${command}`,
                error instanceof Error ? error : undefined
            );
        }
    }

    async checkVersions(manifestPath?: string): Promise<AnalysisReport> {
        try {
            const command = ['check'];
            
            if (manifestPath) {
                command.push('--manifest-path', manifestPath);
            }
            
            command.push('--json-output');
            
            const { stdout } = await this.execute(command.join(' '));
            return JSON.parse(stdout);
        } catch (error) {
            if (error instanceof SyntaxError) {
                throw new VersionTrackingError('Failed to parse version check results');
            }
            throw error;
        }
    }

    async analyzeDependencies(manifestPath?: string, recursive = false): Promise<void> {
        const command = ['analyze'];
        
        if (manifestPath) {
            command.push('--manifest-path', manifestPath);
        }
        
        if (recursive) {
            command.push('--recursive');
        }
        
        await this.execute(command.join(' '));
    }

    async compareFiles(oldFile: string, newFile: string): Promise<any> {
        const { stdout } = await this.execute(`compare "${oldFile}" "${newFile}"`);
        try {
            return JSON.parse(stdout);
        } catch (error) {
            throw new VersionTrackingError('Failed to parse comparison results');
        }
    }

    async searchCrates(query: string, categories: string[] = []): Promise<any> {
        const command = ['search', query];
        if (categories.length > 0) {
            command.push('--categories', categories.join(','));
        }
        
        const { stdout } = await this.execute(command.join(' '));
        try {
            return JSON.parse(stdout);
        } catch (error) {
            throw new VersionTrackingError('Failed to parse search results');
        }
    }

    dispose(): void {
        this.outputChannel.dispose();
    }
}