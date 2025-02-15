import * as vscode from 'vscode';

export interface VersionTrackingConfig {
    executablePath: string;
    cacheDirectory: string;
    enabledFeatures: string[];
    autoCheck: boolean;
    showNotifications: boolean;
}

export class ConfigurationService {
    private static readonly SECTION = 'versionTracking';

    private config: vscode.WorkspaceConfiguration;

    constructor() {
        this.config = vscode.workspace.getConfiguration(ConfigurationService.SECTION);
        this.registerConfigurationChangeListener();
    }

    private registerConfigurationChangeListener(): void {
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration(ConfigurationService.SECTION)) {
                this.config = vscode.workspace.getConfiguration(ConfigurationService.SECTION);
            }
        });
    }

    get executablePath(): string {
        return this.config.get('executablePath') || 'versiontracking';
    }

    get cacheDirectory(): string {
        return this.config.get('cacheDirectory') || '';
    }

    get enabledFeatures(): string[] {
        return this.config.get('enabledFeatures') || [];
    }

    get autoCheck(): boolean {
        return this.config.get('autoCheck') || false;
    }

    get showNotifications(): boolean {
        return this.config.get('showNotifications') || true;
    }

    async updateSetting<T>(key: keyof VersionTrackingConfig, value: T): Promise<void> {
        await this.config.update(key, value, vscode.ConfigurationTarget.Global);
    }

    isFeatureEnabled(feature: string): boolean {
        return this.enabledFeatures.includes(feature);
    }

    getFullConfiguration(): VersionTrackingConfig {
        return {
            executablePath: this.executablePath,
            cacheDirectory: this.cacheDirectory,
            enabledFeatures: this.enabledFeatures,
            autoCheck: this.autoCheck,
            showNotifications: this.showNotifications
        };
    }
}