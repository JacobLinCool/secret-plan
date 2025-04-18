<script lang="ts">
	import { initializeApp, unlockVault, vault, initDarkMode } from '$lib/stores.svelte';
	import { goto } from '$app/navigation';
	import LoadingOverlay from '$lib/components/LoadingOverlay.svelte';
	import ErrorNotification from '$lib/components/ErrorNotification.svelte';
	import { onMount } from 'svelte';

	// Form state
	let masterPassword = $state('');
	let isSubmitting = $state(false);

	onMount(async () => {
		initDarkMode();

		const vaultExists = await initializeApp();

		// If vault isn't initialized, go to onboarding
		if (!vaultExists) {
			goto('/onboarding');
			return;
		}

		// If vault is already unlocked, go directly to dashboard
		if (!vault.isLocked) {
			goto('/dashboard');
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		isSubmitting = true;

		try {
			const success = await unlockVault(masterPassword);

			if (success) {
				goto('/dashboard');
			}
		} finally {
			isSubmitting = false;
		}
	}
</script>

<LoadingOverlay />
<ErrorNotification />

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 dark:bg-gray-900">
	<div class="w-full max-w-md">
		<div class="rounded-2xl bg-white p-8 shadow-xl dark:bg-gray-800">
			<div class="mb-8 text-center">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Secret Plan</h1>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
					Unlock your secure password vault
				</p>
			</div>

			<form onsubmit={handleSubmit}>
				<div class="mb-6">
					<label
						for="master-password"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Master Password
					</label>
					<input
						id="master-password"
						type="password"
						bind:value={masterPassword}
						required
						placeholder="Enter your master password"
						class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
					/>
				</div>

				<button
					type="submit"
					disabled={isSubmitting || !masterPassword}
					class="w-full rounded-lg bg-blue-600 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none disabled:opacity-70 dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
				>
					{isSubmitting ? 'Unlocking...' : 'Unlock Vault'}
				</button>

				<div class="mt-6 text-center">
					<p class="text-sm text-gray-600 dark:text-gray-400">
						Lost your master password? Unfortunately,<br />
						there's no way to recover your encrypted data.
					</p>
				</div>
			</form>
		</div>
	</div>
</div>
