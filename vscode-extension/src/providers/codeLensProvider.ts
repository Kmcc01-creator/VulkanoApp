import * as vscode from 'vscode';
import { DependencyInfo } from '../services/versionTracking';

interface CargoTomlDependency {
    name: string;
    line: number;
    version: string;
}

export class DependencyCodeLensProvider implements vscode.CodeLensProvider {
    private _onDidChangeCodeLenses: vscode.EventEmitter<void> = new vscode.EventEmitter<void>();
    public readonly onDidChangeCodeLenses: vscode.Event<void> = this._onDidChangeCodeLenses.event;
    
    private dependencies: Map<string, DependencyInfo> = new Map();

    constructor() {
        // Watch for changes in Cargo.toml files
        const watcher = vscode.workspace.createFileSystemWatcher('**/Cargo.toml');
        watcher.onDidChange(() => this._onDidChangeCodeLenses.fire());
        watcher.onDidCreate(() => this._onDidChangeCodeLenses.fire());
        watcher.onDidDelete(() => this._onDidChangeCodeLenses.fire());
    }

    updateDependencies(deps: DependencyInfo[]): void {
        this.dependencies.clear();
        deps.forEach(dep => this.dependencies.set(dep.name, dep));
        this._onDidChangeCodeLenses.fire();
    }

    private findDependencies(document: vscode.TextDocument): CargoTomlDependency[] {
        const text = document.getText();
        const lines = text.split('\n');
        const dependencies: CargoTomlDependency[] = [];
        let inDependenciesSection = false;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();
            
            // Check for dependencies section
            if (line === '[dependencies]') {
                inDependenciesSection = true;
                continue;
            }
            
            // Exit dependencies section
            if (inDependenciesSection && line.startsWith('[') && line !== '[dependencies]') {
                inDependenciesSection = false;
                continue;
            }

            if (inDependenciesSection && line) {
                // Match both simple "dep = "version"" and table format "dep = { version = "version" }"
                const simpleMatch = line.match(/^"?([^"=\s]+)"?\s*=\s*"([^"]+)"/);
                const tableMatch = line.match(/^"?([^"=\s]+)"?\s*=\s*{\s*version\s*=\s*"([^"]+)"/);
                
                if (simpleMatch || tableMatch) {
                    const match = simpleMatch || tableMatch;
                    if (match) {
                        dependencies.push({
                            name: match[1],
                            version: match[2],
                            line: i
                        });
                    }
                }
            }
        }

        return dependencies;
    }

    async provideCodeLenses(document: vscode.TextDocument): Promise<vscode.CodeLens[]> {
        const codeLenses: vscode.CodeLens[] = [];
        const deps = this.findDependencies(document);

        for (const dep of deps) {
            const line = document.lineAt(dep.line);
            const range = new vscode.Range(
                new vscode.Position(dep.line, 0),
                line.range.end
            );

            const dependencyInfo = this.dependencies.get(dep.name);
            
            if (dependencyInfo?.updateRecommended) {
                // Update available
                codeLenses.push(
                    new vscode.CodeLens(range, {
                        title: `Update to ${dependencyInfo.latestVersion}`,
                        command: 'versionTracking.updateDependency',
                        arguments: [document.uri, dep.name, dependencyInfo.latestVersion]
                    })
                );
            }

            if (dependencyInfo?.vulnerabilities.length) {
                // Security vulnerabilities
                codeLenses.push(
                    new vscode.CodeLens(range, {
                        title: `${dependencyInfo.vulnerabilities.length} vulnerabilities`,
                        command: 'versionTracking.showVulnerabilities',
                        arguments: [dep.name, dependencyInfo.vulnerabilities]
                    })
                );
            }

            // Show details lens
            codeLenses.push(
                new vscode.CodeLens(range, {
                    title: 'Show Details',
                    command: 'versionTracking.showDependencyDetails',
                    arguments: [dep.name]
                })
            );
        }

        // Add global "Update All" lens if there are any updates available
        if (deps.length > 0 && Array.from(this.dependencies.values()).some(d => d.updateRecommended)) {
            codeLenses.push(
                new vscode.CodeLens(new vscode.Range(0, 0, 0, 0), {
                    title: 'Update All Dependencies',
                    command: 'versionTracking.updateAllDependencies',
                    arguments: [document.uri]
                })
            );
        }

        return codeLenses;
    }
}

export function registerCodeLensProvider(): DependencyCodeLensProvider {
    const provider = new DependencyCodeLensProvider();
    
    vscode.languages.registerCodeLensProvider(
        { language: 'toml', pattern: '**/Cargo.toml' },
        provider
    );

    return provider;
}