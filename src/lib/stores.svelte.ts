import { invoke } from '@tauri-apps/api/core';
import type { BreachState, Credential, Secret } from './types';

// App state management
// Following proper Svelte 5 patterns for sharing state between modules
// Using objects that can be mutated rather than primitive exports

// Vault status
export const vault = $state({
	isLocked: true,
	isInitialized: false
});

// Current credentials list
export const credentialStore = $state({
	items: [] as Credential[]
});

// Search and filter state
export const filters = $state({
	searchTerm: '',
	selectedTag: null as string | null,
	minStrength: null as number | null,
	breachFilter: null as number | null
});

// Loading state
export const ui = $state({
	isLoading: false,
	loadingMessage: '',
	errorMessage: '',
	showError: false,
	darkMode: window.matchMedia?.('(prefers-color-scheme: dark)').matches || false,
	notification: null as null | {
		type: 'error' | 'success' | 'info';
		title?: string;
		message: string;
	}
});

// Initialize app and check vault status
export async function initializeApp() {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Initializing vault...';

		const vaultExists = await invoke<boolean>('initialize_vault');
		vault.isInitialized = vaultExists;

		if (vaultExists) {
			vault.isLocked = await invoke<boolean>('is_vault_locked');
		}

		return vaultExists;
	} catch (error) {
		setError(`Failed to initialize app: ${error}`);
		return false;
	} finally {
		ui.isLoading = false;
	}
}

// Create a new vault
export async function createVault(masterPassword: string) {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Creating vault...';

		await invoke('create_vault', { masterPassword });
		vault.isLocked = false;
		vault.isInitialized = true;

		return true;
	} catch (error) {
		setError(`Failed to create vault: ${error}`);
		return false;
	} finally {
		ui.isLoading = false;
	}
}

// Unlock the vault
export async function unlockVault(masterPassword: string) {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Unlocking vault...';

		const success = await invoke<boolean>('unlock_vault', { masterPassword });

		if (success) {
			vault.isLocked = false;
			await loadCredentials();
			return true;
		} else {
			setError('Incorrect master password');
			return false;
		}
	} catch (error) {
		setError(`Failed to unlock vault: ${error}`);
		return false;
	} finally {
		ui.isLoading = false;
	}
}

// Lock the vault
export async function lockVault() {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Locking vault...';

		await invoke('lock_vault');
		vault.isLocked = true;
		credentialStore.items = [];

		return true;
	} catch (error) {
		setError(`Failed to lock vault: ${error}`);
		return false;
	} finally {
		ui.isLoading = false;
	}
}

// Load credentials with optional filters
export async function loadCredentials() {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Loading credentials...';

		credentialStore.items = await invoke<Credential[]>('search_credentials', {
			searchTerm: filters.searchTerm || undefined,
			tag: filters.selectedTag || undefined,
			minStrength: filters.minStrength || undefined,
			breachState: filters.breachFilter
		});

		return credentialStore.items;
	} catch (error) {
		setError(`Failed to load credentials: ${error}`);
		return [];
	} finally {
		ui.isLoading = false;
	}
}

// Get a credential's secret
export async function getCredentialSecret(uuid: string): Promise<Secret | null> {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Loading secret...';

		return await invoke<Secret>('get_credential_secret', { uuid });
	} catch (error) {
		setError(`Failed to get credential secret: ${error}`);
		return null;
	} finally {
		ui.isLoading = false;
	}
}

// Add a new credential
export async function addCredential(
	site: string,
	username: string,
	password: string,
	notes?: string,
	totp?: string,
	customFields?: Record<string, string>,
	tags?: string
) {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Adding credential...';

		const newCredential = await invoke<Credential>('add_credential', {
			site,
			username,
			password,
			notes,
			totp,
			customFields: customFields ? JSON.stringify(customFields) : undefined
		});

		// If tags were provided, update the credential with tags
		if (tags && tags.trim() !== '') {
			// TODO: Update tags once implemented in backend
		}

		// Refresh the credentials list
		await loadCredentials();

		return newCredential;
	} catch (error) {
		setError(`Failed to add credential: ${error}`);
		return null;
	} finally {
		ui.isLoading = false;
	}
}

// Delete a credential
export async function deleteCredential(uuid: string) {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Deleting credential...';

		await invoke('delete_credential', { uuid });

		// Refresh the credentials list
		await loadCredentials();

		return true;
	} catch (error) {
		setError(`Failed to delete credential: ${error}`);
		return false;
	} finally {
		ui.isLoading = false;
	}
}

// Check if a password has been breached
export async function checkPasswordBreach(uuid: string) {
	try {
		ui.isLoading = true;
		ui.loadingMessage = 'Checking for breaches...';

		const breachState = await invoke<BreachState>('check_password_breach', { uuid });

		// Refresh the credentials list to get updated breach state
		await loadCredentials();

		return breachState;
	} catch (error) {
		setError(`Failed to check password breach: ${error}`);
		return null;
	} finally {
		ui.isLoading = false;
	}
}

// Generate a password
export async function generatePassword(
	length: number = 16,
	useUppercase: boolean = true,
	useLowercase: boolean = true,
	useNumbers: boolean = true,
	useSymbols: boolean = true,
	excludeSimilar: boolean = false
) {
	try {
		return await invoke<string>('generate_password', {
			length,
			useUppercase,
			useLowercase,
			useNumbers,
			useSymbols,
			excludeSimilar
		});
	} catch (error) {
		setError(`Failed to generate password: ${error}`);
		return null;
	}
}

// Error handling helper
function setError(message: string) {
	ui.errorMessage = message;
	ui.showError = true;
	console.error(message);

	// Auto-hide after 5 seconds
	setTimeout(() => {
		ui.showError = false;
	}, 5000);
}

// Toggle dark mode
export function toggleDarkMode() {
	ui.darkMode = !ui.darkMode;

	// Apply dark mode to document
	if (ui.darkMode) {
		document.documentElement.classList.add('dark');
	} else {
		document.documentElement.classList.remove('dark');
	}
}

// Initialize dark mode based on system preference
export function initDarkMode() {
	if (ui.darkMode) {
		document.documentElement.classList.add('dark');
	}
}
