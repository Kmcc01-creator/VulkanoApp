import * as vscode from 'vscode';
import { DependencyInfo } from '../services/versionTracking';

export class DependencyTreeItem extends vscode.TreeItem {
    constructor(
        public readonly dependency: DependencyInfo,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(dependency.name, collapsibleState);
        
        const hasUpdate = dependency.latestVersion && dependency.updateRecommended;
        const hasVulnerabilities = dependency.vulnerabilities.length > 0;
        
        this.description = `${dependency.currentVersion} ${
            hasUpdate ? `→ ${dependency.latestVersion}` : ''
        }`;
        
        const tooltip = new vscode.MarkdownString();
        tooltip.appendMarkdown(`**${dependency.name}**\n\n`);
        tooltip.appendMarkdown(`Current: ${dependency.currentVersion}\n`);
        if (dependency.latestVersion) {
            tooltip.appendMarkdown(`Latest: ${dependency.latestVersion}\n`);
        }
        if (hasVulnerabilities) {
            tooltip.appendMarkdown('\n**Vulnerabilities:**\n');
            dependency.vulnerabilities.forEach(v => {
                tooltip.appendMarkdown(`- ${v}\n`);
            });
        }
        this.tooltip = tooltip;

        if (hasVulnerabilities) {
            this.iconPath = new vscode.ThemeIcon('warning', new vscode.ThemeColor('errorForeground'));
        } else if (hasUpdate) {
            this.iconPath = new vscode.ThemeIcon('arrow-up', new vscode.ThemeColor('notificationCenterHeader.background'));
        } else {
            this.iconPath = new vscode.ThemeIcon('check', new vscode.ThemeColor('terminal.ansiGreen'));
        }

        this.contextValue = [
            'dependency',
            hasUpdate ? 'hasUpdate' : '',
            hasVulnerabilities ? 'hasVulnerabilities' : ''
        ].filter(Boolean).join('-');
    }
}

export class DependencyTreeProvider implements vscode.TreeDataProvider<DependencyTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<DependencyTreeItem | undefined | null | void> =
        new vscode.EventEmitter<DependencyTreeItem | undefined | null | void>();

    readonly onDidChangeTreeData: vscode.Event<DependencyTreeItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    private dependencies: DependencyInfo[] = [];

    constructor() {
        this.dependencies = [];
    }

    getTreeItem(element: DependencyTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: DependencyTreeItem): Thenable<DependencyTreeItem[]> {
        if (element) {
            // For now, dependencies don't have children
            return Promise.resolve([]);
        }

        // Root level - show all dependencies
        return Promise.resolve(
            this.dependencies.map(
                dep =>
                    new DependencyTreeItem(
                        dep,
                        vscode.TreeItemCollapsibleState.None
                    )
            )
        );
    }

    refresh(dependencies: DependencyInfo[]): void {
        this.dependencies = dependencies;
        this._onDidChangeTreeData.fire();
    }

    getDependencyCount(): number {
        return this.dependencies.length;
    }

    getUpdateCount(): number {
        return this.dependencies.filter(dep => dep.updateRecommended).length;
    }

    getVulnerabilityCount(): number {
        return this.dependencies.filter(dep => dep.vulnerabilities.length > 0).length;
    }
}

export function registerDependencyTreeProvider(): DependencyTreeProvider {
    const treeDataProvider = new DependencyTreeProvider();
    vscode.window.registerTreeDataProvider('versionTracking.dependenciesView', treeDataProvider);
    return treeDataProvider;
}