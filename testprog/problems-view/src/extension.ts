import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

class ProblemsTreeProvider implements vscode.TreeDataProvider<ProblemItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ProblemItem | undefined | null | void> = new vscode.EventEmitter<ProblemItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ProblemItem | undefined | null | void> = this._onDidChangeTreeData.event;
    private problemsChannel: vscode.OutputChannel;

    constructor() {
        this.problemsChannel = vscode.window.createOutputChannel("Problems Channel");
        
        // Update tree when diagnostics change
        vscode.languages.onDidChangeDiagnostics(() => {
            this.refresh();
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ProblemItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ProblemItem): Promise<ProblemItem[]> {
        if (!element) {
            // Root level - show files with problems
            const problems = vscode.languages.getDiagnostics();
            return problems
                .filter(([_, diagnostics]) => diagnostics.length > 0)
                .map(([uri, diagnostics]) => {
                    const fileName = path.basename(uri.fsPath);
                    return new ProblemItem(
                        fileName,
                        uri,
                        vscode.TreeItemCollapsibleState.Collapsed,
                        {
                            title: `${diagnostics.length} problems`,
                            tooltip: uri.fsPath,
                            iconPath: this.getFileIcon(diagnostics)
                        }
                    );
                });
        } else {
            // Show problems for the selected file
            const problems = vscode.languages.getDiagnostics(element.resourceUri!);
            return problems.map(diagnostic => {
                const line = diagnostic.range.start.line + 1;
                const col = diagnostic.range.start.character + 1;
                return new ProblemItem(
                    `${line}:${col} - ${diagnostic.message}`,
                    element.resourceUri!,
                    vscode.TreeItemCollapsibleState.None,
                    {
                        severity: diagnostic.severity,
                        command: {
                            command: 'problems-view.openProblemLocation',
                            title: 'Go to Problem',
                            arguments: [element.resourceUri!, diagnostic.range]
                        }
                    }
                );
            });
        }
    }

    private getFileIcon(diagnostics: vscode.Diagnostic[]): vscode.ThemeIcon {
        const hasError = diagnostics.some(d => d.severity === vscode.DiagnosticSeverity.Error);
        const hasWarning = diagnostics.some(d => d.severity === vscode.DiagnosticSeverity.Warning);
        if (hasError) return new vscode.ThemeIcon('error');
        if (hasWarning) return new vscode.ThemeIcon('warning');
        return new vscode.ThemeIcon('info');
    }

    async exportProblems(): Promise<void> {
        const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        
        if (!workspacePath) {
            throw new Error('No workspace folder found');
        }

        const outputFile = path.join(workspacePath, 'problemschan.txt');
        const diagnostics = vscode.languages.getDiagnostics();
        let output = '';

        // Process all diagnostics
        for (const [uri, diagnosticArray] of diagnostics) {
            if (diagnosticArray.length > 0) {
                output += `\nFile: ${uri.fsPath}\n`;
                
                for (const diagnostic of diagnosticArray) {
                    const line = diagnostic.range.start.line + 1;
                    const col = diagnostic.range.start.character + 1;
                    const severity = getSeverityLabel(diagnostic.severity);
                    const message = `[${severity}] Line ${line}:${col} - ${diagnostic.message}`;
                    
                    // Write to output channel
                    this.problemsChannel.appendLine(message);
                    
                    // Add to output string for file
                    output += message + '\n';
                }
            }
        }

        // Show the output channel in VS Code
        this.problemsChannel.show();

        // Write to file asynchronously
        await fs.writeFile(outputFile, output, 'utf8');
        
        // Log to terminal
        console.log(output);

        return;
    }
}

class ProblemItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly resourceUri: vscode.Uri,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        options: {
            severity?: vscode.DiagnosticSeverity;
            command?: vscode.Command;
            title?: string;
            tooltip?: string;
            iconPath?: vscode.ThemeIcon;
        } = {}
    ) {
        super(label, collapsibleState);
        this.tooltip = options.tooltip || label;
        this.description = options.title;
        this.command = options.command;
        if (options.iconPath) {
            this.iconPath = options.iconPath;
        } else if (options.severity !== undefined) {
            this.iconPath = this.getSeverityIcon(options.severity);
        }
    }

    private getSeverityIcon(severity: vscode.DiagnosticSeverity): vscode.ThemeIcon {
        switch (severity) {
            case vscode.DiagnosticSeverity.Error:
                return new vscode.ThemeIcon('error');
            case vscode.DiagnosticSeverity.Warning:
                return new vscode.ThemeIcon('warning');
            case vscode.DiagnosticSeverity.Information:
                return new vscode.ThemeIcon('info');
            case vscode.DiagnosticSeverity.Hint:
                return new vscode.ThemeIcon('lightbulb');
            default:
                return new vscode.ThemeIcon('question');
        }
    }
}

function getSeverityLabel(severity: vscode.DiagnosticSeverity): string {
    switch (severity) {
        case vscode.DiagnosticSeverity.Error:
            return 'Error';
        case vscode.DiagnosticSeverity.Warning:
            return 'Warning';
        case vscode.DiagnosticSeverity.Information:
            return 'Info';
        case vscode.DiagnosticSeverity.Hint:
            return 'Hint';
        default:
            return 'Unknown';
    }
}

export function activate(context: vscode.ExtensionContext) {
    const problemsProvider = new ProblemsTreeProvider();

    // Register the tree view
    const treeView = vscode.window.createTreeView('problemsExplorer', {
        treeDataProvider: problemsProvider,
        showCollapseAll: true
    });

    // Register command to open problem location
    context.subscriptions.push(
        vscode.commands.registerCommand('problems-view.openProblemLocation', (uri: vscode.Uri, range: vscode.Range) => {
            vscode.window.showTextDocument(uri, {
                selection: range,
                preserveFocus: false,
                preview: false
            });
        })
    );

    // Register refresh command
    context.subscriptions.push(
        vscode.commands.registerCommand('problems-view.refresh', () => {
            problemsProvider.refresh();
        })
    );

    // Register export problems command
    context.subscriptions.push(
        vscode.commands.registerCommand('problems-view.exportProblems', async () => {
            try {
                await problemsProvider.exportProblems();
                vscode.window.showInformationMessage('Problems exported successfully');
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to export problems: ${error}`);
            }
        })
    );

    context.subscriptions.push(treeView);
}

export function deactivate() {}