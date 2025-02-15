import * as vscode from 'vscode';
import { DependencyInfo } from '../services/versionTracking';

export class DependencyHoverProvider implements vscode.HoverProvider {
    private dependencies: Map<string, DependencyInfo> = new Map();

    constructor() {}

    updateDependencies(deps: DependencyInfo[]): void {
        this.dependencies.clear();
        deps.forEach(dep => this.dependencies.set(dep.name, dep));
    }

    async provideHover(
        document: vscode.TextDocument,
        position: vscode.Position
    ): Promise<vscode.Hover | null> {
        const line = document.lineAt(position.line).text;
        
        // Check if we're in the dependencies section
        const textBeforeLine = document.getText(new vscode.Range(0, 0, position.line, 0));
        if (!this.isInDependenciesSection(textBeforeLine)) {
            return null;
        }

        // Try to extract dependency name from the line
        const depName = this.extractDependencyName(line);
        if (!depName) {
            return null;
        }

        const dependencyInfo = this.dependencies.get(depName);
        if (!dependencyInfo) {
            return null;
        }

        return this.createHover(dependencyInfo);
    }

    private isInDependenciesSection(text: string): boolean {
        const lines = text.split('\n');
        let inDependencies = false;

        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed === '[dependencies]') {
                inDependencies = true;
            } else if (trimmed.startsWith('[') && trimmed !== '[dependencies]') {
                inDependencies = false;
            }
        }

        return inDependencies;
    }

    private extractDependencyName(line: string): string | null {
        // Match both simple "dep = "version"" and table format "dep = { version = "version" }"
        const simpleMatch = line.match(/^"?([^"=\s]+)"?\s*=/);
        if (simpleMatch) {
            return simpleMatch[1];
        }
        return null;
    }

    private createHover(dependency: DependencyInfo): vscode.Hover {
        const content = new vscode.MarkdownString();
        
        // Header with current version
        content.appendMarkdown(`## ${dependency.name} \`v${dependency.currentVersion}\`\n\n`);
        
        // Version information
        if (dependency.latestVersion) {
            if (dependency.updateRecommended) {
                content.appendMarkdown(`⚠️ Update available: \`v${dependency.latestVersion}\`\n\n`);
            } else {
                content.appendMarkdown(`✅ Up to date\n\n`);
            }
        }

        // Vulnerabilities
        if (dependency.vulnerabilities.length > 0) {
            content.appendMarkdown(`### Security Vulnerabilities\n\n`);
            dependency.vulnerabilities.forEach(vuln => {
                content.appendMarkdown(`🔓 ${vuln}\n`);
            });
            content.appendMarkdown('\n');
        }

        // Make links clickable
        content.isTrusted = true;
        
        // Add commands
        content.appendMarkdown('---\n\n');
        if (dependency.updateRecommended) {
            content.appendMarkdown(`[Update to v${dependency.latestVersion}](command:versionTracking.updateDependency?${encodeURIComponent(JSON.stringify([dependency.name, dependency.latestVersion]))})\n\n`);
        }
        content.appendMarkdown(`[View Details](command:versionTracking.showDependencyDetails?${encodeURIComponent(JSON.stringify([dependency.name]))})`);

        return new vscode.Hover(content);
    }
}

export function registerHoverProvider(): DependencyHoverProvider {
    const provider = new DependencyHoverProvider();
    
    vscode.languages.registerHoverProvider(
        { language: 'toml', pattern: '**/Cargo.toml' },
        provider
    );

    return provider;
}