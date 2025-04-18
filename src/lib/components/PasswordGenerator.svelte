<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { ui } from '../stores.svelte';

	interface Props {
		onSelect?: (password: string) => void;
	}
	let { onSelect }: Props = $props();

	// Password generation options
	let length = $state(16);
	let useUppercase = $state(true);
	let useLowercase = $state(true);
	let useNumbers = $state(true);
	let useSymbols = $state(true);
	let excludeSimilar = $state(true);
	let generatedPassword = $state('');

	// Generate password on mount and when options change
	$effect(() => {
		generatePassword();
	});

	// Generate a new password
	async function generatePassword() {
		try {
			ui.isLoading = true;

			// Use Tauri backend to generate password
			const password = (await invoke('generate_password', {
				length,
				useUppercase,
				useLowercase,
				useNumbers,
				useSymbols,
				excludeSimilar
			})) as string;

			generatedPassword = password;
		} catch (error) {
			console.error('Failed to generate password:', error);
			ui.notification = {
				type: 'error',
				message: 'Failed to generate password'
			};
		} finally {
			ui.isLoading = false;
		}
	}

	// Strength calculation (simplified version, should use the backend version later)
	function calculatePasswordStrength() {
		if (!generatedPassword) return { score: 0, color: 'bg-gray-200' };

		// Basic entropy calculation
		let score = 0;
		let entropy = 0;

		if (useUppercase) entropy += 26;
		if (useLowercase) entropy += 26;
		if (useNumbers) entropy += 10;
		if (useSymbols) entropy += 32;

		// Entropy bits = log2(charsetSize) * length
		const entropyBits = Math.log2(entropy) * length;

		if (entropyBits > 128) score = 100;
		else if (entropyBits > 80) score = 80;
		else if (entropyBits > 60) score = 60;
		else if (entropyBits > 40) score = 40;
		else score = 20;

		let color = 'bg-red-500';
		if (score >= 80) color = 'bg-green-500';
		else if (score >= 60) color = 'bg-yellow-500';
		else if (score >= 40) color = 'bg-orange-500';

		return { score, color };
	}

	const passwordStrength = $derived(calculatePasswordStrength());

	// Copy password to clipboard
	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(generatedPassword);
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

	// Select this password (for parent component)
	function handleSelect() {
		onSelect?.(generatedPassword);
	}
</script>

<div class="rounded-lg bg-white p-4 shadow dark:bg-gray-800">
	<h3 class="mb-4 text-lg font-medium text-gray-900 dark:text-white">Password Generator</h3>

	<div class="mb-4">
		<div class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
			Length: {length}
		</div>
		<input
			type="range"
			id="password-length"
			bind:value={length}
			min="8"
			max="32"
			step="1"
			class="h-2 w-full cursor-pointer appearance-none rounded-lg bg-gray-200 dark:bg-gray-700"
			aria-label="Password Length"
		/>
	</div>

	<div class="mb-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
		<label class="flex items-center">
			<input
				type="checkbox"
				bind:checked={useUppercase}
				class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
			/>
			<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Uppercase (A-Z)</span>
		</label>

		<label class="flex items-center">
			<input
				type="checkbox"
				bind:checked={useLowercase}
				class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
			/>
			<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Lowercase (a-z)</span>
		</label>

		<label class="flex items-center">
			<input
				type="checkbox"
				bind:checked={useNumbers}
				class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
			/>
			<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Numbers (0-9)</span>
		</label>

		<label class="flex items-center">
			<input
				type="checkbox"
				bind:checked={useSymbols}
				class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
			/>
			<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Symbols (!@#$%...)</span>
		</label>

		<label class="col-span-full flex items-center">
			<input
				type="checkbox"
				bind:checked={excludeSimilar}
				class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
			/>
			<span class="ml-2 text-sm text-gray-700 dark:text-gray-300"
				>Exclude similar characters (l, 1, I, O, 0)</span
			>
		</label>
	</div>

	<div class="mb-4">
		<div class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
			Generated Password
		</div>
		<div class="flex">
			<input
				type="text"
				id="generated-password"
				readonly
				value={generatedPassword}
				class="block w-full rounded-l-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
				aria-label="Generated Password"
			/>
			<button
				onclick={copyToClipboard}
				class="inline-flex items-center rounded-r-lg border border-l-0 border-gray-300 bg-gray-200 px-3 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-600 dark:text-gray-300"
				aria-label="Copy to clipboard"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
					/>
				</svg>
			</button>
		</div>
	</div>

	<div class="mb-4">
		<div class="mb-1 flex justify-between">
			<span class="text-xs font-medium text-gray-500 dark:text-gray-400">Password Strength</span>
		</div>
		<div class="h-1.5 w-full rounded-full bg-gray-200 dark:bg-gray-700">
			<div
				class="h-1.5 rounded-full {passwordStrength.color}"
				style="width: {passwordStrength.score}%"
			></div>
		</div>
	</div>

	<div class="flex space-x-2">
		<button
			onclick={generatePassword}
			class="rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:hover:bg-gray-700 dark:focus:ring-gray-700"
		>
			Regenerate
		</button>

		<button
			onclick={handleSelect}
			class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
		>
			Use This Password
		</button>
	</div>
</div>
