import type React from 'react';

import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { useSettingsStore, useShallow } from '@/store';
import { Wallet, Lock, Eye, EyeOff, Check, X, TrendingUp, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';

export default function SettingsPage() {
    const settings = useSettingsStore(
        state => ({
            databaseUrl: state.databaseUrl,
            sentrySdn: state.sentrySdn,
            claudeApiKey: state.claudeApiKey,
            openaiApiKey: state.openaiApiKey,
            llmProvider: state.llmProvider,
            twitterBearerToken: state.twitterBearerToken,
            paperTradingEnabled: state.paperTradingEnabled,
            paperTradingBalance: state.paperTradingBalance,
            buyInAmounts: state.buyInAmounts,
            defaultBuyInAmount: state.defaultBuyInAmount,
            phantomConnected: state.phantomConnected,
            phantomAddress: state.phantomAddress,
            theme: state.theme,
            updateSetting: state.updateSetting,
            togglePaperTrading: state.togglePaperTrading,
            addBuyInPreset: state.addBuyInPreset,
            removeBuyInPreset: state.removeBuyInPreset,
            connectPhantom: state.connectPhantom,
            disconnectPhantom: state.disconnectPhantom,
            resetSettings: state.resetSettings,
        }),
        useShallow
    );

    const [showPasswords, setShowPasswords] = useState<Record<string, boolean>>({});
    const [saveSuccess, setSaveSuccess] = useState(false);
    const [paperTradingInput, setPaperTradingInput] = useState(
        settings.paperTradingBalance.toString()
    );
    const [newBuyInAmount, setNewBuyInAmount] = useState('');

    const togglePasswordVisibility = (field: string) => {
        setShowPasswords(prev => ({
            ...prev,
            [field]: !prev[field],
        }));
    };

    const handleConnectPhantom = async () => {
        try {
            const response = await (window as any).solana?.connect();
            if (response?.publicKey) {
                settings.connectPhantom(response.publicKey.toString());
                setSaveSuccess(true);
                setTimeout(() => setSaveSuccess(false), 3000);
            }
        } catch (error) {
            console.error('Failed to connect Phantom wallet:', error);
            alert("Please install Phantom wallet or check if it's enabled");
        }
    };

    const handleDisconnectPhantom = async () => {
        try {
            await (window as any).solana?.disconnect();
            settings.disconnectPhantom();
        } catch (error) {
            console.error('Failed to disconnect Phantom wallet:', error);
        }
    };

    const handleSaveAPIKeys = () => {
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    const handleClearAPIKeys = () => {
        if (window.confirm('Are you sure you want to clear all API keys? This cannot be undone.')) {
            settings.resetSettings();
            setSaveSuccess(true);
            setTimeout(() => setSaveSuccess(false), 3000);
        }
    };

    const handlePaperTradingToggle = () => {
        settings.togglePaperTrading();
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    const handlePaperTradingBalanceChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const value = e.target.value;
        setPaperTradingInput(value);
    };

    const handleSavePaperTradingBalance = () => {
        const balance = Number.parseFloat(paperTradingInput);
        if (isNaN(balance) || balance < 0) {
            alert('Please enter a valid amount');
            return;
        }
        settings.updateSetting('paperTradingBalance', balance);
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    const handleResetPaperTrading = () => {
        if (window.confirm('Reset paper trading balance to $10,000?')) {
            settings.updateSetting('paperTradingBalance', 10000);
            setPaperTradingInput('10000');
            setSaveSuccess(true);
            setTimeout(() => setSaveSuccess(false), 3000);
        }
    };

    const handleAddBuyInAmount = () => {
        const amount = Number.parseFloat(newBuyInAmount);
        if (isNaN(amount) || amount <= 0) {
            alert('Please enter a valid amount');
            return;
        }
        if (settings.buyInAmounts.includes(amount)) {
            alert('This amount already exists');
            return;
        }
        settings.addBuyInPreset(amount);
        setNewBuyInAmount('');
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    const handleRemoveBuyInAmount = (amount: number) => {
        settings.removeBuyInPreset(amount);
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    const handleSetDefaultBuyIn = (amount: number) => {
        settings.updateSetting('defaultBuyInAmount', amount);
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
    };

    return (
        <div className="p-6 space-y-6 fade-in">
            <div>
                <h1 className="text-3xl font-bold text-foreground">Settings</h1>
                <p className="text-muted-foreground mt-1">Configure your account, wallets, and API keys</p>
            </div>

            {/* Success Message */}
            {saveSuccess && (
                <div className="flex items-center gap-2 px-4 py-3 bg-accent/20 border border-accent rounded-lg text-accent">
                    <Check className="w-4 h-4" />
                    <span className="text-sm font-medium">Settings saved successfully</span>
                </div>
            )}

            {/* Account Settings */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle>Account Settings</CardTitle>
                    <CardDescription>Manage your account information</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div>
                        <label className="text-sm font-medium text-foreground">Email</label>
                        <input
                            type="email"
                            className="w-full bg-input rounded px-3 py-2 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary"
                            defaultValue="user@eclipse.com"
                        />
                    </div>
                    <div>
                        <label className="text-sm font-medium text-foreground">Username</label>
                        <input
                            type="text"
                            className="w-full bg-input rounded px-3 py-2 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary"
                            defaultValue="trader"
                        />
                    </div>
                    <button
                        onClick={handleSaveAPIKeys}
                        className="w-full bg-primary text-primary-foreground rounded py-2 font-medium hover:opacity-90 transition-opacity"
                    >
                        Save Changes
                    </button>
                </CardContent>
            </Card>

            {/* Phantom Wallet */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Wallet className="w-5 h-5 text-accent" />
                        Phantom Wallet
                    </CardTitle>
                    <CardDescription>Connect your Phantom wallet for seamless trading</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {settings.phantomConnected ? (
                        <div className="bg-accent/10 border border-accent rounded-lg p-4">
                            <div className="flex items-center justify-between">
                                <div>
                                    <p className="text-sm font-medium text-foreground">Connected Wallet</p>
                                    <p className="text-xs text-muted-foreground font-mono mt-1">
                                        {settings.phantomAddress}
                                    </p>
                                </div>
                                <div className="px-3 py-1 bg-accent/20 text-accent rounded text-xs font-medium flex items-center gap-1">
                                    <Check className="w-3 h-3" />
                                    Connected
                                </div>
                            </div>
                            <button
                                onClick={handleDisconnectPhantom}
                                className="w-full mt-4 px-4 py-2 border border-destructive text-destructive rounded hover:bg-destructive/10 transition-colors font-medium"
                            >
                                Disconnect Wallet
                            </button>
                        </div>
                    ) : (
                        <div className="bg-muted/10 border border-border rounded-lg p-4">
                            <p className="text-sm text-muted-foreground mb-4">
                                No wallet connected. Connect your Phantom wallet to enable:
                            </p>
                            <ul className="text-xs text-muted-foreground space-y-1 mb-4 ml-4">
                                <li>• Direct wallet interactions</li>
                                <li>• Seamless trading execution</li>
                                <li>• Portfolio synchronization</li>
                            </ul>
                            <button
                                onClick={handleConnectPhantom}
                                className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity font-medium"
                            >
                                <Wallet className="w-4 h-4" />
                                Connect Phantom Wallet
                            </button>
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Buy-In Preferences */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <TrendingUp className="w-5 h-5 text-accent" />
                        Buy-In Preferences
                    </CardTitle>
                    <CardDescription>
                        Customize your quick buy-in amounts when jumping into new coins
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                    {/* Add new buy-in amount */}
                    <div>
                        <label className="text-sm font-medium text-foreground block mb-2">
                            Add Quick Buy-In Amount
                        </label>
                        <div className="flex gap-2">
                            <div className="relative flex-1">
                                <span className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground">
                                    $
                                </span>
                                <input
                                    type="number"
                                    value={newBuyInAmount}
                                    onChange={e => setNewBuyInAmount(e.target.value)}
                                    placeholder="Enter amount"
                                    min="1"
                                    step="5"
                                    className="w-full bg-input rounded px-3 py-2 pl-7 text-foreground border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-sm"
                                />
                            </div>
                            <button
                                onClick={handleAddBuyInAmount}
                                className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity font-medium flex items-center gap-2"
                            >
                                <Plus className="w-4 h-4" />
                                Add
                            </button>
                        </div>
                    </div>

                    {/* Display current buy-in amounts */}
                    <div>
                        <label className="text-sm font-medium text-foreground block mb-3">
                            Your Quick Buy-In Amounts
                        </label>
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                            {settings.buyInAmounts.length === 0 ? (
                                <p className="text-xs text-muted-foreground col-span-full">
                                    No buy-in amounts configured. Add one to get started.
                                </p>
                            ) : (
                                settings.buyInAmounts.map(amount => (
                                    <div
                                        key={amount}
                                        className={`flex items-center justify-between px-3 py-2 rounded border transition-colors ${settings.defaultBuyInAmount === amount
                                                ? 'bg-accent/20 border-accent'
                                                : 'bg-muted/10 border-border hover:border-primary'
                                            }`}
                                    >
                                        <div className="flex items-center gap-2 flex-1">
                                            <input
                                                type="radio"
                                                name="default-buyin"
                                                checked={settings.defaultBuyInAmount === amount}
                                                onChange={() => handleSetDefaultBuyIn(amount)}
                                                className="w-4 h-4 cursor-pointer"
                                            />
                                            <span className="text-sm font-medium text-foreground">${amount}</span>
                                            {settings.defaultBuyInAmount === amount && (
                                                <span className="text-xs text-accent font-semibold ml-auto">Default</span>
                                            )}
                                        </div>
                                        <button
                                            onClick={() => handleRemoveBuyInAmount(amount)}
                                            className="ml-2 p-1 hover:bg-destructive/10 text-destructive rounded transition-colors"
                                        >
                                            <Trash2 className="w-4 h-4" />
                                        </button>
                                    </div>
                                ))
                            )}
                        </div>
                    </div>

                    {/* Info */}
                    <div className="bg-muted/10 border border-border rounded-lg p-3">
                        <p className="text-xs text-muted-foreground">
                            <span className="font-semibold text-foreground">Buy-In Info:</span> These amounts
                            appear as quick buttons when viewing new coins in the Market Surveillance. Select a
                            default amount that will be pre-selected when jumping into coins.
                        </p>
                    </div>
                </CardContent>
            </Card>

            {/* Paper Trading */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <TrendingUp className="w-5 h-5 text-accent" />
                        Paper Trading
                    </CardTitle>
                    <CardDescription>Practice trading with virtual money without real risk</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="flex items-center justify-between p-4 bg-muted/10 rounded-lg border border-border">
                        <div>
                            <p className="text-sm font-medium text-foreground">Paper Trading Mode</p>
                            <p className="text-xs text-muted-foreground mt-1">
                                {settings.paperTradingEnabled
                                    ? 'Active - Trading with virtual funds'
                                    : 'Disabled - Enable to start practicing'}
                            </p>
                        </div>
                        <button
                            onClick={handlePaperTradingToggle}
                            className={`px-4 py-2 rounded font-medium transition-colors ${settings.paperTradingEnabled
                                    ? 'bg-accent text-accent-foreground hover:opacity-90'
                                    : 'bg-muted text-muted-foreground hover:bg-muted/80'
                                }`}
                        >
                            {settings.paperTradingEnabled ? 'Deactivate' : 'Activate'}
                        </button>
                    </div>

                    {settings.paperTradingEnabled && (
                        <div className="space-y-4 pt-2">
                            <div>
                                <label className="text-sm font-medium text-foreground block mb-2">
                                    Starting Balance
                                </label>
                                <div className="flex gap-2">
                                    <div className="relative flex-1">
                                        <span className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground">
                                            $
                                        </span>
                                        <input
                                            type="number"
                                            value={paperTradingInput}
                                            onChange={handlePaperTradingBalanceChange}
                                            placeholder="10000"
                                            min="0"
                                            step="100"
                                            className="w-full bg-input rounded px-3 py-2 pl-7 text-foreground border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-sm"
                                        />
                                    </div>
                                    <button
                                        onClick={handleSavePaperTradingBalance}
                                        className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity font-medium"
                                    >
                                        Set Balance
                                    </button>
                                </div>
                            </div>

                            <div className="bg-accent/10 border border-accent rounded-lg p-4">
                                <p className="text-sm font-medium text-foreground mb-1">Current Virtual Balance</p>
                                <p className="text-2xl font-bold text-accent">
                                    $
                                    {settings.paperTradingBalance.toLocaleString('en-US', {
                                        minimumFractionDigits: 2,
                                        maximumFractionDigits: 2,
                                    })}
                                </p>
                                <button
                                    onClick={handleResetPaperTrading}
                                    className="w-full mt-3 px-4 py-2 border border-accent text-accent rounded hover:bg-accent/10 transition-colors font-medium text-sm"
                                >
                                    Reset to $10,000
                                </button>
                            </div>

                            <div className="bg-muted/10 border border-border rounded-lg p-3">
                                <p className="text-xs text-muted-foreground">
                                    <span className="font-semibold text-foreground">Paper Trading Info:</span> Your
                                    virtual balance is used for simulating trades. This does not affect your real
                                    portfolio or wallet.
                                </p>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* API Keys Management */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Lock className="w-5 h-5 text-accent" />
                        API Keys & Configuration
                    </CardTitle>
                    <CardDescription>
                        Add your own API keys instead of relying on .env file. These are stored locally in your
                        browser.
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                    {/* Database URL */}
                    <div>
                        <label className="text-sm font-medium text-foreground flex items-center gap-2">
                            Database URL
                            <span className="text-xs text-muted-foreground font-normal">(SQLite)</span>
                        </label>
                        <input
                            type="text"
                            value={settings.databaseUrl}
                            onChange={e => settings.updateSetting('databaseUrl', e.target.value)}
                            placeholder="sqlite:./database.db"
                            className="w-full bg-input rounded px-3 py-2 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-xs"
                        />
                    </div>

                    {/* Sentry DSN */}
                    <div>
                        <label className="text-sm font-medium text-foreground flex items-center gap-2">
                            Sentry DSN
                            <span className="text-xs text-muted-foreground font-normal">(Optional)</span>
                        </label>
                        <input
                            type="text"
                            value={settings.sentrySdn}
                            onChange={e => settings.updateSetting('sentrySdn', e.target.value)}
                            placeholder="https://your-key@sentry.io/project-id"
                            className="w-full bg-input rounded px-3 py-2 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-xs"
                        />
                    </div>

                    {/* LLM Provider Selection */}
                    <div>
                        <label className="text-sm font-medium text-foreground block mb-2">LLM Provider</label>
                        <div className="grid grid-cols-2 gap-2">
                            {['claude', 'gpt4'].map(provider => (
                                <button
                                    key={provider}
                                    onClick={() =>
                                        settings.updateSetting('llmProvider', provider as 'claude' | 'gpt4')
                                    }
                                    className={`py-2 px-3 rounded border transition-colors ${settings.llmProvider === provider
                                            ? 'bg-primary border-primary text-primary-foreground'
                                            : 'bg-muted/10 border-border text-foreground hover:border-primary'
                                        }`}
                                >
                                    {provider === 'claude' ? 'Claude (Anthropic)' : 'GPT-4 (OpenAI)'}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Claude API Key */}
                    <div>
                        <label className="text-sm font-medium text-foreground flex items-center gap-2">
                            Claude API Key
                            <span className="text-xs text-muted-foreground font-normal">(Anthropic)</span>
                        </label>
                        <div className="relative">
                            <input
                                type={showPasswords['claude'] ? 'text' : 'password'}
                                value={settings.claudeApiKey}
                                onChange={e => settings.updateSetting('claudeApiKey', e.target.value)}
                                placeholder="sk-ant-..."
                                className="w-full bg-input rounded px-3 py-2 pr-10 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-xs"
                            />
                            <button
                                onClick={() => togglePasswordVisibility('claude')}
                                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                            >
                                {showPasswords['claude'] ? (
                                    <EyeOff className="w-4 h-4" />
                                ) : (
                                    <Eye className="w-4 h-4" />
                                )}
                            </button>
                        </div>
                    </div>

                    {/* OpenAI API Key */}
                    <div>
                        <label className="text-sm font-medium text-foreground flex items-center gap-2">
                            OpenAI API Key
                            <span className="text-xs text-muted-foreground font-normal">(GPT-4)</span>
                        </label>
                        <div className="relative">
                            <input
                                type={showPasswords['openai'] ? 'text' : 'password'}
                                value={settings.openaiApiKey}
                                onChange={e => settings.updateSetting('openaiApiKey', e.target.value)}
                                placeholder="sk-proj-..."
                                className="w-full bg-input rounded px-3 py-2 pr-10 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-xs"
                            />
                            <button
                                onClick={() => togglePasswordVisibility('openai')}
                                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                            >
                                {showPasswords['openai'] ? (
                                    <EyeOff className="w-4 h-4" />
                                ) : (
                                    <Eye className="w-4 h-4" />
                                )}
                            </button>
                        </div>
                    </div>

                    {/* Twitter Bearer Token */}
                    <div>
                        <label className="text-sm font-medium text-foreground flex items-center gap-2">
                            Twitter Bearer Token
                            <span className="text-xs text-muted-foreground font-normal">(X API v2)</span>
                        </label>
                        <div className="relative">
                            <input
                                type={showPasswords['twitter'] ? 'text' : 'password'}
                                value={settings.twitterBearerToken}
                                onChange={e => settings.updateSetting('twitterBearerToken', e.target.value)}
                                placeholder="AAAA..."
                                className="w-full bg-input rounded px-3 py-2 pr-10 text-foreground mt-1 border border-border focus:outline-none focus:ring-2 focus:ring-primary font-mono text-xs"
                            />
                            <button
                                onClick={() => togglePasswordVisibility('twitter')}
                                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                            >
                                {showPasswords['twitter'] ? (
                                    <EyeOff className="w-4 h-4" />
                                ) : (
                                    <Eye className="w-4 h-4" />
                                )}
                            </button>
                        </div>
                    </div>

                    {/* Action Buttons */}
                    <div className="pt-4 border-t border-border space-y-2">
                        <button
                            onClick={handleSaveAPIKeys}
                            className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity font-medium"
                        >
                            <Check className="w-4 h-4" />
                            Save API Keys
                        </button>
                        <button
                            onClick={handleClearAPIKeys}
                            className="w-full flex items-center justify-center gap-2 px-4 py-2 border border-destructive text-destructive rounded hover:bg-destructive/10 transition-colors font-medium"
                        >
                            <X className="w-4 h-4" />
                            Clear All Keys
                        </button>
                    </div>

                    {/* Security Notice */}
                    <div className="bg-muted/10 border border-border rounded-lg p-3">
                        <p className="text-xs text-muted-foreground">
                            <span className="font-semibold text-foreground">Security Note:</span> Your API keys
                            are stored locally in your browser's localStorage. They are never sent to our servers.
                            Clear your browser data to remove them.
                        </p>
                    </div>
                </CardContent>
            </Card>

            {/* Theme & Color Scheme */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle>Theme & Color Scheme</CardTitle>
                    <CardDescription>Customize the look and feel of Eclipse Market Trading</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div>
                        <label className="text-sm font-medium text-foreground block mb-3">Select Theme</label>
                        <div className="grid grid-cols-2 gap-3">
                            {[
                                { id: 'eclipse', label: 'Eclipse', desc: 'Deep purple and gold' },
                                { id: 'midnight', label: 'Midnight', desc: 'Navy and cyan' },
                                { id: 'cyber', label: 'Cyber', desc: 'Electric pink and green' },
                                { id: 'lunar', label: 'Lunar', desc: 'Silver and blue' },
                            ].map(themeOption => (
                                <button
                                    key={themeOption.id}
                                    onClick={() => {
                                        settings.updateSetting(
                                            'theme',
                                            themeOption.id as 'eclipse' | 'midnight' | 'cyber' | 'lunar'
                                        );
                                    }}
                                    className={`p-4 rounded border-2 transition-all text-left ${settings.theme === themeOption.id
                                            ? 'border-accent bg-accent/10'
                                            : 'border-border hover:border-primary'
                                        }`}
                                >
                                    <p className="text-sm font-semibold text-foreground">{themeOption.label}</p>
                                    <p className="text-xs text-muted-foreground">{themeOption.desc}</p>
                                </button>
                            ))}
                        </div>
                    </div>

                    <div className="bg-muted/10 border border-border rounded-lg p-3">
                        <p className="text-xs text-muted-foreground">
                            <span className="font-semibold text-foreground">Theme Note:</span> Changes apply
                            immediately across the entire app. Your theme preference is saved locally.
                        </p>
                    </div>
                </CardContent>
            </Card>

            {/* Preferences */}
            <Card className="bg-card border-border">
                <CardHeader>
                    <CardTitle>Preferences</CardTitle>
                    <CardDescription>Customize your trading experience</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="flex items-center justify-between p-3 rounded bg-muted/10">
                        <label className="text-sm font-medium text-foreground">Dark Mode</label>
                        <input type="checkbox" className="w-4 h-4 cursor-pointer" defaultChecked />
                    </div>
                    <div className="flex items-center justify-between p-3 rounded bg-muted/10">
                        <label className="text-sm font-medium text-foreground">Push Notifications</label>
                        <input type="checkbox" className="w-4 h-4 cursor-pointer" defaultChecked />
                    </div>
                    <div className="flex items-center justify-between p-3 rounded bg-muted/10">
                        <label className="text-sm font-medium text-foreground">Email Alerts</label>
                        <input type="checkbox" className="w-4 h-4 cursor-pointer" defaultChecked />
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
