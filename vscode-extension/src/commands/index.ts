import * as vscode from 'vscode';
import { VersionTrackingClient } from '../services/versionTracking';
import { ProgressService } from '../services/progress';
import { ConfigurationService } from '../services/configuration';
import { DependencyTreeProvider } from '../providers/dependencyTreeProvider';
import { DependencyCodeLensProvider } from '../providers/codeLensProvider';
import { DependencyHoverProvider } from '../providers/hoverProvider';

export class CommandManager {
    constructor(
        private client: VersionTrackingClient,
        private config: ConfigurationService,
        private progress: ProgressService,
        private treeProvider: DependencyTreeProvider,
        private codeLensProvider: DependencyCodeLensProvider,
        private hoverProvider: DependencyHoverProvider
    ) {}

    registerCommands(context: vscode.ExtensionContext): void {
        context.subscriptions.push(
            vscode.commands.registerCommand(
                'versionTracking.checkDependencies',
                () => this.checkDependencies()
            ),
            vscode.commands.registerCommand(
                'versionTracking.updateDependency',
                (uri: vscode.Uri, name: string, version: string) => this.updateDependency(uri, name, version)
            ),
            vscode.commands.registerCommand(
                'versionTracking.updateAllDependencies',
                (uri: vscode.Uri) => this.updateAllDependencies(uri)
            ),
            vscode.commands.registerCommand(
                'versionTracking.showDependencyDetails',
                (name: string) => this.showDependencyDetails(name)
            ),
            vscode.commands.registerCommand(
                'versionTracking.showVulnerabilities',
                (name: string, vulnerabilities: string[]) => this.showVulnerabilities(name, vulnerabilities)
            ),
            vscode.commands.registerCommand(
                'versionTracking.refreshDependencies',
                () => this.refreshDependencies()
            )
        );
    }

    private async checkDependencies(): Promise<void> {
        try {
            await this.progress.withProgress(
                { title: 'Checking dependencies...' },
                async (progress) => {
                    progress.report({ message: 'Analyzing project...' });
                    
                    const report = await this.client.checkVersions(
                        vscode.workspace.workspaceFolders?.[0].uri.fsPath
                    );
                    
                    // Update all providers with new dependency information
                    this.treeProvider.refresh(report.dependencies);
                    this.codeLensProvider.updateDependencies(report.dependencies);
                    this.hoverProvider.updateDependencies(report.dependencies);

                    const updates = report.dependencies.filter(d => d.updateRecommended).length;
                    const vulnerabilities = report.dependencies.filter(d => d.vulnerabilities.length > 0).length;

                    if (updates || vulnerabilities) {
                        vscode.window.showWarningMessage(
                            `Found ${updates} updates and ${vulnerabilities} vulnerabilities.`
                        );
                    } else {
                        vscode.window.showInformationMessage('All dependencies are up to date!');
                    }
                }
            );
        } catch (error) {
            vscode.window.showErrorMessage(
                `Failed to check dependencies: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    private async updateDependency(uri: vscode.Uri, name: string, version: string): Promise<void> {
        try {
            await this.progress.withProgress(
                { title: `Updating ${name}...` },
                async () => {
                    // TODO: Implement actual dependency update logic
                    await vscode.window.showInformationMessage(
                        `Updating ${name} to version ${version}`
                    );
                    await this.refreshDependencies();
                }
            );
        } catch (error) {
            vscode.window.showErrorMessage(
                `Failed to update ${name}: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    private async updateAllDependencies(uri: vscode.Uri): Promise<void> {
        try {
            await this.progress.withProgress(
                { title: 'Updating all dependencies...' },
                async () => {
                    // TODO: Implement actual bulk update logic
                    await vscode.window.showInformationMessage(
                        'Updating all dependencies'
                    );
                    await this.refreshDependencies();
                }
            );
        } catch (error) {
            vscode.window.showErrorMessage(
                `Failed to update dependencies: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    private async showDependencyDetails(name: string): Promise<void> {
        try {
            // TODO: Implement detailed dependency view
            await vscode.window.showInformationMessage(
                `Showing details for ${name}`
            );
        } catch (error) {
            vscode.window.showErrorMessage(
                `Failed to show details: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    private async showVulnerabilities(name: string, vulnerabilities: string[]): Promise<void> {
        const message = new vscode.MarkdownString(`## Security Vulnerabilities in ${name}\n\n`);
        vulnerabilities.forEach(v => {
            message.appendMarkdown(`- 🔓 ${v}\n`);
        });

        const panel = vscode.window.createWebviewPanel(
            'vulnerabilities',
            `Vulnerabilities: ${name}`,
            vscode.ViewColumn.One,
            {}
        );

        panel.webview.html = `
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body { padding: 10px; }
                    .vulnerability { margin: 10px 0; padding: 10px; background: #fff3f3; border-left: 4px solid #ff0000; }
                </style>
            </head>
            <body>
                <h2>Security Vulnerabilities in ${name}</h2>
                ${vulnerabilities.map(v => `<div class="vulnerability">${v}</div>`).join('')}
            </body>
            </html>
        `;
    }

    private async refreshDependencies(): Promise<void> {
        await this.checkDependencies();
    }
}