import * as vscode from 'vscode';
import { VersionTrackingClient } from './services/versionTracking';
import { ConfigurationService } from './services/configuration';
import { ProgressService } from './services/progress';
import { registerDependencyTreeProvider } from './providers/dependencyTreeProvider';
import { registerCodeLensProvider } from './providers/codeLensProvider';
import { registerHoverProvider } from './providers/hoverProvider';
import { CommandManager } from './commands';

export async function activate(context: vscode.ExtensionContext) {
    // Initialize services
    const config = new ConfigurationService();
    const progress = new ProgressService();
    const client = new VersionTrackingClient();

    // Initialize providers
    const treeProvider = registerDependencyTreeProvider();
    const codeLensProvider = registerCodeLensProvider();
    const hoverProvider = registerHoverProvider();

    // Initialize command manager
    const commandManager = new CommandManager(
        client,
        config,
        progress,
        treeProvider,
        codeLensProvider,
        hoverProvider
    );

    // Register commands
    commandManager.registerCommands(context);

    // Create status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Left
    );
    statusBarItem.text = "$(package) Dependencies";
    statusBarItem.tooltip = "Check Project Dependencies";
    statusBarItem.command = 'versionTracking.checkDependencies';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Set up auto-check on Cargo.toml changes if enabled
    if (config.autoCheck) {
        const watcher = vscode.workspace.createFileSystemWatcher('**/Cargo.toml');
        watcher.onDidChange(async () => {
            if (config.autoCheck) {
                await vscode.commands.executeCommand('versionTracking.checkDependencies');
            }
        });
        context.subscriptions.push(watcher);
    }

    // Register configuration change handler
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('versionTracking')) {
                // Refresh providers when configuration changes
                vscode.commands.executeCommand('versionTracking.refreshDependencies');
            }
        })
    );

    // Set up diagnostic collection for showing problems
    const diagnostics = vscode.languages.createDiagnosticCollection('versionTracking');
    context.subscriptions.push(diagnostics);

    // Initial dependency check
    await vscode.commands.executeCommand('versionTracking.checkDependencies');

    console.log('Version Tracking extension is now active');
}

export function deactivate() {
    // Clean up resources if needed
}
