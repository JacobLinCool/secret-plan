<script lang="ts">
	import { addCredential } from '../stores.svelte';
	import PasswordGenerator from './PasswordGenerator.svelte';

	interface Props {
		onClose: () => void;
	}
	let { onClose }: Props = $props();

	// Form state
	let site = $state('');
	let username = $state('');
	let password = $state('');
	let notes = $state('');
	let tags = $state('');
	let showPasswordGenerator = $state(false);
	let isSubmitting = $state(false);

	// Form submission
	async function handleSubmit(e: Event) {
		e.preventDefault();
		isSubmitting = true;

		try {
			await addCredential(
				site,
				username,
				password,
				notes || undefined,
				undefined, // TOTP not implemented in UI yet
				undefined, // Custom fields not implemented in UI yet
				tags
			);

			// Clear form and close
			resetForm();
			onClose();
		} catch (error) {
			console.error('Failed to add credential:', error);
		} finally {
			isSubmitting = false;
		}
	}

	// Reset the form
	function resetForm() {
		site = '';
		username = '';
		password = '';
		notes = '';
		tags = '';
	}

	// Handle generated password selection
	function useGeneratedPassword(newPassword: string) {
		password = newPassword;
		showPasswordGenerator = false;
	}

	// Cancel form
	function handleCancel() {
		resetForm();
		onClose();
	}
</script>

<div
	class="mx-auto max-w-2xl rounded-lg border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
>
	<div class="mb-6 flex items-center justify-between">
		<h2 class="text-xl font-semibold text-gray-900 dark:text-white">Add New Credential</h2>
		<button
			onclick={handleCancel}
			class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
			aria-label="Close"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M6 18L18 6M6 6l12 12"
				/>
			</svg>
		</button>
	</div>

	<form onsubmit={handleSubmit}>
		<div class="mb-4">
			<label for="site" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
				Website/Application
			</label>
			<input
				id="site"
				type="text"
				bind:value={site}
				required
				placeholder="example.com"
				class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
			/>
		</div>

		<div class="mb-4">
			<label for="username" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
				Username/Email
			</label>
			<input
				id="username"
				type="text"
				bind:value={username}
				required
				placeholder="user@example.com"
				class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
			/>
		</div>

		<div class="mb-4">
			<label for="password" class="mb-2 flex items-center justify-between">
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">Password</span>
				<button
					type="button"
					onclick={() => {
						showPasswordGenerator = !showPasswordGenerator;
					}}
					class="text-xs font-medium text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
				>
					{showPasswordGenerator ? 'Hide Generator' : 'Generate Password'}
				</button>
			</label>
			<input
				id="password"
				type="password"
				bind:value={password}
				required
				placeholder="••••••••"
				class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
			/>
		</div>

		{#if showPasswordGenerator}
			<div class="mb-4">
				<PasswordGenerator onSelect={useGeneratedPassword} />
			</div>
		{/if}

		<div class="mb-4">
			<label for="notes" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
				Notes (Optional)
			</label>
			<textarea
				id="notes"
				bind:value={notes}
				rows="3"
				class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
				placeholder="Additional notes..."
			></textarea>
		</div>

		<div class="mb-4">
			<label for="tags" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
				Tags (Optional, comma-separated)
			</label>
			<input
				id="tags"
				type="text"
				bind:value={tags}
				placeholder="work, finance, social"
				class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
			/>
		</div>

		<div class="mt-6 flex justify-end space-x-4">
			<button
				type="button"
				onclick={handleCancel}
				class="rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
			>
				Cancel
			</button>
			<button
				type="submit"
				disabled={isSubmitting}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none disabled:opacity-70 dark:bg-blue-700 dark:hover:bg-blue-800 dark:focus:ring-blue-800"
			>
				{isSubmitting ? 'Adding...' : 'Add Credential'}
			</button>
		</div>
	</form>
</div>
