import * as vscode from 'vscode';

export interface ProgressOptions {
    title: string;
    cancellable?: boolean;
    location?: vscode.ProgressLocation;
}

export class ProgressService {
    private activeProgress?: vscode.Progress<{ message?: string; increment?: number }>;
    private activeToken?: vscode.CancellationTokenSource;

    async withProgress<T>(
        options: ProgressOptions,
        task: (
            progress: vscode.Progress<{ message?: string; increment?: number }>,
            token: vscode.CancellationToken
        ) => Promise<T>
    ): Promise<T> {
        this.activeToken?.dispose();
        this.activeToken = new vscode.CancellationTokenSource();

        return vscode.window.withProgress(
            {
                location: options.location || vscode.ProgressLocation.Notification,
                title: options.title,
                cancellable: options.cancellable
            },
            async (progress, token) => {
                this.activeProgress = progress;
                token.onCancellationRequested(() => {
                    this.activeToken?.dispose();
                });

                try {
                    return await task(progress, token);
                } finally {
                    this.activeProgress = undefined;
                }
            }
        );
    }

    showIndeterminateProgress(message: string): vscode.Disposable {
        const statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Left
        );

        let dots = '';
        const interval = setInterval(() => {
            dots = dots.length >= 3 ? '' : dots + '.';
            statusBarItem.text = `$(sync~spin) ${message}${dots}`;
        }, 500);

        statusBarItem.show();

        return {
            dispose: () => {
                clearInterval(interval);
                statusBarItem.dispose();
            }
        };
    }

    async progressStep(message: string, increment?: number): Promise<void> {
        if (this.activeProgress) {
            this.activeProgress.report({ message, increment });
        }
    }

    cancelCurrentProgress(): void {
        if (this.activeToken) {
            this.activeToken.cancel();
            this.activeToken = undefined;
        }
    }

    static async withNotification<T>(
        title: string,
        task: () => Promise<T>
    ): Promise<T> {
        const progressService = new ProgressService();
        return progressService.withProgress(
            {
                title,
                location: vscode.ProgressLocation.Notification
            },
            async (progress) => {
                progress.report({ message: 'Starting...' });
                return task();
            }
        );
    }
}