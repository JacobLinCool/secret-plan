<script lang="ts">
	import { createVault, vault, initDarkMode } from '$lib/stores.svelte';
	import { goto } from '$app/navigation';
	import LoadingOverlay from '$lib/components/LoadingOverlay.svelte';
	import ErrorNotification from '$lib/components/ErrorNotification.svelte';
	import PasswordGenerator from '$lib/components/PasswordGenerator.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		initDarkMode();
	});

	// Form state
	let masterPassword = $state('');
	let confirmPassword = $state('');
	let showPasswordGenerator = $state(false);
	let rememberMaster = $state(false);
	let currentStep = $state(0);

	// Password validation - fixed by properly defining function instead of using derived
	function getPasswordStrength() {
		if (!masterPassword) return { score: 0, label: 'Empty', color: 'bg-gray-200' };

		if (masterPassword.length < 8) {
			return { score: 1, label: 'Weak', color: 'bg-red-500' };
		} else if (masterPassword.length < 12) {
			return { score: 2, label: 'Fair', color: 'bg-orange-500' };
		} else if (masterPassword.length < 16) {
			return { score: 3, label: 'Good', color: 'bg-yellow-500' };
		} else {
			return { score: 4, label: 'Strong', color: 'bg-green-500' };
		}
	}

	const passwordStrength = $derived(getPasswordStrength());

	const passwordsMatch = $derived(!confirmPassword || masterPassword === confirmPassword);

	// Use generated password
	function useGeneratedPassword(password: string) {
		masterPassword = password;
		confirmPassword = password;
		showPasswordGenerator = false;
	}

	// Form submission
	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!masterPassword || masterPassword !== confirmPassword) {
			return;
		}

		const success = await createVault(masterPassword);

		if (success) {
			// Navigate to dashboard
			goto('/dashboard');
		}
	}

	// Check if vault already exists, if so redirect to login
	$effect(() => {
		if (vault.isInitialized) {
			goto('/');
		}
	});
</script>

<LoadingOverlay />
<ErrorNotification />

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 dark:bg-gray-900">
	<div class="w-full max-w-md">
		<div class="rounded-2xl bg-white p-8 shadow-xl dark:bg-gray-800">
			<div class="mb-6 text-center">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Welcome to Secret Plan</h1>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
					Your zero-knowledge password manager
				</p>
			</div>

			<div class="mb-6">
				<div class="relative">
					<div
						class="absolute top-1/2 left-0 h-0.5 w-full -translate-y-1/2 transform bg-gray-200 dark:bg-gray-700"
					></div>
					<div class="relative flex justify-between">
						<div
							class="z-10 flex h-6 w-6 items-center justify-center rounded-full bg-blue-600 text-xs font-medium text-white"
						>
							1
						</div>
						<div
							class="z-10 flex h-6 w-6 items-center justify-center rounded-full {currentStep >= 1
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400'}"
						>
							2
						</div>
						<div
							class="z-10 flex h-6 w-6 items-center justify-center rounded-full {currentStep >= 2
								? 'bg-blue-600 text-white'
								: 'bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400'}"
						>
							3
						</div>
					</div>
				</div>
			</div>

			{#if currentStep === 0}
				<div>
					<h2 class="text-lg font-medium text-gray-900 dark:text-white">Create Master Password</h2>
					<p class="mt-1 text-sm text-gray-600 dark:text-gray-400">
						This password will be used to encrypt all your secrets. Make sure it's strong and
						memorable.
					</p>

					<div class="mt-4">
						<label
							for="master-password"
							class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
							>Master Password</label
						>
						<input
							id="master-password"
							type="password"
							bind:value={masterPassword}
							placeholder="Enter your master password"
							class="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
						/>

						<div class="mt-2">
							<div class="mb-1 flex justify-between">
								<span class="text-xs font-medium text-gray-500 dark:text-gray-400"
									>Password Strength</span
								>
								<span class="text-xs font-medium text-gray-500 dark:text-gray-400"
									>{passwordStrength.label}</span
								>
							</div>
							<div class="h-1.5 w-full rounded-full bg-gray-200 dark:bg-gray-700">
								<div
									class="h-1.5 rounded-full {passwordStrength.color}"
									style="width: {passwordStrength.score * 25}%"
								></div>
							</div>
						</div>

						<div class="mt-4 flex items-center justify-end">
							<button
								type="button"
								onclick={() => {
									showPasswordGenerator = !showPasswordGenerator;
								}}
								class="text-sm font-medium text-blue-600 hover:text-blue-700 dark:text-blue-500 dark:hover:text-blue-400"
							>
								{showPasswordGenerator ? 'Hide Generator' : 'Generate Strong Password'}
							</button>
						</div>

						{#if showPasswordGenerator}
							<div class="mt-4">
								<PasswordGenerator onSelect={useGeneratedPassword} />
							</div>
						{/if}

						<div class="mt-4">
							<label
								for="confirm-password"
								class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
								>Confirm Password</label
							>
							<input
								id="confirm-password"
								type="password"
								bind:value={confirmPassword}
								placeholder="Confirm your master password"
								class="block w-full rounded-lg border {!passwordsMatch
									? 'border-red-500'
									: 'border-gray-300'} bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
							/>
							{#if !passwordsMatch && confirmPassword}
								<p class="mt-1 text-xs text-red-500">Passwords do not match</p>
							{/if}
						</div>
					</div>

					<div class="mt-6">
						<button
							onclick={() => {
								if (masterPassword && passwordsMatch && getPasswordStrength().score >= 2)
									currentStep = 1;
							}}
							disabled={!masterPassword || !passwordsMatch || passwordStrength.score < 2}
							class="w-full rounded-lg bg-blue-600 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none disabled:opacity-50 dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
						>
							Continue
						</button>
					</div>
				</div>
			{:else if currentStep === 1}
				<div>
					<h2 class="text-lg font-medium text-gray-900 dark:text-white">
						Important Security Notice
					</h2>
					<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
						Your master password is the key to all your secrets. Please note:
					</p>

					<ul class="mt-4 list-inside list-disc space-y-2 text-sm text-gray-600 dark:text-gray-400">
						<li>We cannot recover your master password if you lose it</li>
						<li>Your data is encrypted/decrypted locally on your device</li>
						<li>Never share your master password with anyone</li>
						<li>Use a password that's easy for you to remember but hard for others to guess</li>
					</ul>

					<div class="mt-6">
						<label for="remember-master" class="flex items-center">
							<input
								id="remember-master"
								type="checkbox"
								bind:checked={rememberMaster}
								class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
							/>
							<span class="ml-2 text-sm text-gray-700 dark:text-gray-300"
								>I have safely stored my master password</span
							>
						</label>
					</div>

					<div class="mt-6 flex space-x-4">
						<button
							onclick={() => {
								currentStep = 0;
							}}
							class="flex-1 rounded-lg border border-gray-300 bg-white px-5 py-2.5 text-center text-sm font-medium text-gray-700 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:hover:bg-gray-700 dark:focus:ring-gray-700"
						>
							Back
						</button>
						<button
							onclick={() => {
								if (rememberMaster) currentStep = 2;
							}}
							disabled={!rememberMaster}
							class="flex-1 rounded-lg bg-blue-600 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none disabled:opacity-50 dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
						>
							Continue
						</button>
					</div>
				</div>
			{:else}
				<div>
					<h2 class="text-lg font-medium text-gray-900 dark:text-white">Create Your Vault</h2>
					<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
						You're all set! Click below to create your secure vault with Argon2 key-derivation and
						AES-256-GCM encryption.
					</p>

					<form onsubmit={handleSubmit} class="mt-6">
						<button
							type="submit"
							class="w-full rounded-lg bg-blue-600 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
						>
							Create Vault
						</button>

						<button
							type="button"
							onclick={() => {
								currentStep = 1;
							}}
							class="mt-4 w-full text-center text-sm text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200"
						>
							Back
						</button>
					</form>
				</div>
			{/if}
		</div>
	</div>
</div>
