<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { credentialStore, ui } from '../stores.svelte';
	import type { Credential, Secret } from '../types';

	interface Props {
		credential: Credential;
	}
	let { credential }: Props = $props();

	// Local state
	let isExpanded = $state(false);
	let isLoading = $state(false);
	let decryptedSecret = $state<Secret | null>(null);
	let showDeleteConfirm = $state(false);
	let showPassword = $state(false);

	// Toggle expanded card and load secret
	async function toggleExpand() {
		if (isExpanded) {
			// Collapse
			isExpanded = false;
			decryptedSecret = null;
			showPassword = false;
		} else {
			// Expand and fetch secret
			isExpanded = true;
			await loadSecret();
		}
	}

	// Load the decrypted secret
	async function loadSecret() {
		if (decryptedSecret) return; // Already loaded

		try {
			isLoading = true;

			const secret = (await invoke('get_credential_secret', {
				uuid: credential.uuid
			})) as Secret;

			decryptedSecret = secret;
		} catch (error) {
			console.error('Failed to load secret:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to decrypt password'
			};
		} finally {
			isLoading = false;
		}
	}

	// Delete this credential
	async function deleteCredential() {
		try {
			ui.isLoading = true;

			await invoke('delete_credential', {
				uuid: credential.uuid
			});

			// Remove from store
			credentialStore.items = credentialStore.items.filter((c) => c.uuid !== credential.uuid);

			ui.notification = {
				type: 'success',
				message: 'Credential deleted successfully'
			};
		} catch (error) {
			console.error('Failed to delete credential:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to delete credential'
			};
		} finally {
			ui.isLoading = false;
			showDeleteConfirm = false;
		}
	}

	// Check for breaches
	async function checkBreaches() {
		try {
			ui.isLoading = true;
			ui.loadingMessage = 'Checking for breaches...';

			const breachState = (await invoke('check_password_breach', {
				uuid: credential.uuid
			})) as number;

			// Update credential in store
			credential.breach_state = breachState;

			const status = breachState === 1 ? 'Safe' : breachState === 2 ? 'Compromised' : 'Unknown';
			ui.notification = {
				type: breachState === 1 ? 'success' : breachState === 2 ? 'error' : 'info',
				title: 'Breach Check Result',
				message: `Password is ${status}`
			};
		} catch (error) {
			console.error('Failed to check breaches:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to check for breaches'
			};
		} finally {
			ui.isLoading = false;
			ui.loadingMessage = '';
		}
	}

	// Copy password to clipboard
	async function copyPassword() {
		if (!decryptedSecret) await loadSecret();
		if (!decryptedSecret) return;

		try {
			await navigator.clipboard.writeText(decryptedSecret.password);
			ui.notification = {
				type: 'success',
				message: 'Password copied to clipboard'
			};
		} catch (error) {
			console.error('Failed to copy password:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to copy password'
			};
		}
	}

	// Copy username to clipboard
	async function copyUsername() {
		try {
			await navigator.clipboard.writeText(credential.username);
			ui.notification = {
				type: 'success',
				message: 'Username copied to clipboard'
			};
		} catch (error) {
			console.error('Failed to copy username:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to copy username'
			};
		}
	}

	// Helper for breach status
	function breachStatus() {
		switch (credential.breach_state) {
			case 0:
				return { label: 'Unknown', color: 'bg-gray-500' };
			case 1:
				return { label: 'Safe', color: 'bg-green-500' };
			case 2:
				return { label: 'Compromised', color: 'bg-red-500' };
			default:
				return { label: 'Unknown', color: 'bg-gray-500' };
		}
	}

	// Helper for strength meter
	function strengthMeter() {
		const strength = credential.strength || 0;
		let color = 'bg-red-500';

		if (strength >= 80) color = 'bg-green-500';
		else if (strength >= 60) color = 'bg-yellow-500';
		else if (strength >= 40) color = 'bg-orange-500';

		return { strength, color };
	}

	// Format tags as array
	function getTags(): string[] {
		return credential.tags || [];
	}

	const tags = $derived(getTags());
	const meterValues = $derived(strengthMeter());
	const breach = $derived(breachStatus());
</script>

<div
	class="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800"
>
	<!-- Card Header -->
	<div class="p-4">
		<div class="flex items-start justify-between">
			<div>
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">
					{credential.site}
				</h3>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					{credential.username}
				</p>
			</div>
			<div class="flex space-x-2">
				{#if credential.breach_state === 2}
					<span
						class="inline-flex items-center rounded-full bg-red-100 px-2.5 py-0.5 text-xs font-medium text-red-800 dark:bg-red-900/20 dark:text-red-300"
					>
						<span class="mr-1 h-2 w-2 rounded-full bg-red-500"></span>
						Compromised
					</span>
				{/if}
				<button
					onclick={toggleExpand}
					aria-label="Expand"
					class="inline-flex rounded p-1 text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5 transition-transform {isExpanded ? 'rotate-180' : ''}"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
							clip-rule="evenodd"
						/>
					</svg>
				</button>
			</div>
		</div>

		<!-- Tags -->
		{#if tags.length > 0}
			<div class="mt-2 flex flex-wrap gap-1">
				{#each tags as tag}
					<span
						class="rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>{tag}</span
					>
				{/each}
			</div>
		{/if}

		<!-- Strength meter -->
		<div class="mt-3">
			<div class="mb-1 flex justify-between">
				<span class="text-xs font-medium text-gray-500 dark:text-gray-400">Strength</span>
			</div>
			<div class="h-1 w-full rounded-full bg-gray-200 dark:bg-gray-700">
				<div
					class="h-1 rounded-full {meterValues.color}"
					style="width: {meterValues.strength}%"
				></div>
			</div>
		</div>

		<!-- Quick Actions -->
		<div class="mt-3 flex space-x-2">
			<button
				onclick={copyUsername}
				class="inline-flex items-center rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mr-1 h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
					/>
				</svg>
				Copy Username
			</button>
			<button
				onclick={copyPassword}
				class="inline-flex items-center rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mr-1 h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
					/>
				</svg>
				Copy Password
			</button>
		</div>
	</div>

	<!-- Expanded section -->
	{#if isExpanded}
		<div class="border-t border-gray-200 p-4 dark:border-gray-700">
			{#if isLoading}
				<div class="flex items-center justify-center py-4">
					<div
						class="h-5 w-5 animate-spin rounded-full border-2 border-blue-600 border-t-transparent"
					></div>
					<span class="ml-2 text-sm text-gray-600 dark:text-gray-400">Decrypting...</span>
				</div>
			{:else if decryptedSecret}
				<!-- Password -->
				<div class="mb-3">
					<div
						id="password-label"
						class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Password
					</div>
					<div class="flex">
						<input
							type={showPassword ? 'text' : 'password'}
							value={decryptedSecret.password}
							readonly
							aria-labelledby="password-label"
							class="block w-full rounded-l-lg border border-gray-300 bg-gray-50 p-2 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
						/>
						<button
							onclick={() => (showPassword = !showPassword)}
							class="inline-flex items-center rounded-r-lg border border-l-0 border-gray-300 bg-gray-200 px-3 text-sm text-gray-500 dark:border-gray-600 dark:bg-gray-600 dark:text-gray-400"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="h-5 w-5"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								{#if showPassword}
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
									/>
								{:else}
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
									/>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
									/>
								{/if}
							</svg>
						</button>
					</div>
				</div>

				<!-- Notes (if any) -->
				{#if decryptedSecret.notes}
					<div class="mb-3">
						<div
							id="notes-label"
							class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
						>
							Notes
						</div>
						<div
							class="rounded-lg border border-gray-300 bg-gray-50 p-2 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
							aria-labelledby="notes-label"
						>
							{decryptedSecret.notes}
						</div>
					</div>
				{/if}

				<!-- TOTP (if any) - feature placeholder -->
				{#if decryptedSecret.totp}
					<div class="mb-3">
						<div
							id="totp-label"
							class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
						>
							2FA Code
						</div>
						<div class="flex">
							<div
								class="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2 text-center font-mono text-xl font-bold tracking-widest text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
								aria-labelledby="totp-label"
							>
								{decryptedSecret.totp}
							</div>
						</div>
					</div>
				{/if}

				<!-- Custom fields (if any) -->
				{#if decryptedSecret.custom_fields && Object.keys(decryptedSecret.custom_fields).length > 0}
					<div class="mb-3">
						<div
							id="custom-fields-label"
							class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
						>
							Custom Fields
						</div>
						<div class="space-y-2" aria-labelledby="custom-fields-label">
							{#each Object.entries(decryptedSecret.custom_fields) as [key, value]}
								<div class="flex flex-col">
									<span class="text-xs text-gray-500 dark:text-gray-400">{key}</span>
									<span
										class="rounded-lg border border-gray-300 bg-gray-50 p-2 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
										>{value}</span
									>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			{/if}

			<!-- Actions -->
			<div class="flex flex-wrap gap-2">
				<button
					onclick={checkBreaches}
					class="inline-flex items-center rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="mr-1 h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
						/>
					</svg>
					Check Breach
				</button>

				{#if !showDeleteConfirm}
					<button
						onclick={() => (showDeleteConfirm = true)}
						class="inline-flex items-center rounded-lg border border-red-300 bg-white px-3 py-1.5 text-xs font-medium text-red-700 hover:bg-red-100 focus:ring-4 focus:ring-red-200 focus:outline-none dark:border-red-600 dark:bg-gray-800 dark:text-red-400 dark:hover:bg-gray-700 dark:hover:text-red-500 dark:focus:ring-red-900"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="mr-1 h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
							/>
						</svg>
						Delete
					</button>
				{:else}
					<div class="flex items-center space-x-2">
						<span class="text-xs text-gray-700 dark:text-gray-300">Confirm deletion?</span>
						<button
							onclick={deleteCredential}
							class="inline-flex items-center rounded-lg bg-red-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-red-700 focus:ring-4 focus:ring-red-300 focus:outline-none dark:bg-red-700 dark:hover:bg-red-800 dark:focus:ring-red-900"
						>
							Yes
						</button>
						<button
							onclick={() => (showDeleteConfirm = false)}
							class="inline-flex items-center rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
						>
							No
						</button>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
