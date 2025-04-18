<script lang="ts">
	import { addCredential, credentialStore } from '../stores.svelte';
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
	let tags = $state<string[]>([]);
	let currentTagInput = $state('');
	let tagSuggestions = $state<string[]>([]);
	let showSuggestions = $state(false);
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
		tags = [];
		currentTagInput = '';
	}

	// Handle generated password selection
	function useGeneratedPassword(newPassword: string) {
		password = newPassword;
		showPasswordGenerator = false;
	}

	// Tag handling functions
	function addTag(tag: string) {
		tag = tag.trim().toLowerCase();
		if (tag && !tags.includes(tag)) {
			tags = [...tags, tag];
			currentTagInput = '';
		}
	}

	function removeTag(index: number) {
		tags = tags.filter((_, i) => i !== index);
	}

	function handleTagKeydown(e: KeyboardEvent) {
		// Add tag on Enter or comma key
		if (e.key === 'Enter' || e.key === ',') {
			e.preventDefault();
			addTag(currentTagInput);
		} else if (e.key === 'Backspace' && !currentTagInput && tags.length > 0) {
			// Remove last tag when backspace is pressed in an empty input
			removeTag(tags.length - 1);
		}

		// Update suggestions based on input
		updateSuggestions();
	}

	function updateSuggestions() {
		if (!currentTagInput.trim()) {
			showSuggestions = false;
			return;
		}

		// Get unique tags from existing credentials
		const existingTags = new Set<string>();
		credentialStore.items.forEach((cred) => {
			if (cred.tags) {
				cred.tags.forEach((tag) => existingTags.add(tag));
			}
		});

		// Filter suggestions based on input
		tagSuggestions = Array.from(existingTags)
			.filter((tag) => tag.toLowerCase().includes(currentTagInput.toLowerCase()))
			.filter((tag) => !tags.includes(tag))
			.slice(0, 5); // Limit to 5 suggestions

		showSuggestions = tagSuggestions.length > 0;
	}

	function selectSuggestion(suggestion: string) {
		addTag(suggestion);
		showSuggestions = false;
	}

	// Cancel form
	function handleCancel() {
		resetForm();
		onClose();
	}
</script>

<div
	class="mx-auto w-full max-w-2xl rounded-lg border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
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
				Tags (Optional)
			</label>
			<div class="relative">
				<div
					class="flex flex-wrap items-center gap-2 rounded-lg border border-gray-300 bg-gray-50 p-2 dark:border-gray-600 dark:bg-gray-700"
				>
					{#each tags as tag, index (tag)}
						<span
							class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-sm font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>
							{tag}
							<button
								type="button"
								onclick={() => removeTag(index)}
								class="ml-1.5 inline-flex h-4 w-4 items-center justify-center rounded-full hover:bg-blue-200 dark:hover:bg-blue-800"
								aria-label="Remove tag"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-3 w-3"
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
						</span>
					{/each}
					<input
						id="tag-input"
						type="text"
						bind:value={currentTagInput}
						onkeydown={handleTagKeydown}
						oninput={updateSuggestions}
						placeholder={tags.length > 0
							? 'Add another tag...'
							: 'Add tags (e.g., work, finance, personal)'}
						class="min-w-[120px] flex-1 border-0 bg-transparent p-0 text-gray-900 focus:ring-0 focus:outline-none dark:text-white"
					/>
				</div>

				<div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Press Enter or comma to add a tag
				</div>

				{#if showSuggestions}
					<ul
						class="ring-opacity-5 absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black focus:outline-none sm:text-sm dark:bg-gray-700 dark:ring-gray-600"
					>
						{#each tagSuggestions as suggestion}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
							<li
								onclick={() => selectSuggestion(suggestion)}
								class="relative cursor-pointer py-2 pr-9 pl-3 text-gray-900 select-none hover:bg-blue-100 dark:text-white dark:hover:bg-gray-600"
							>
								{suggestion}
							</li>
						{/each}
					</ul>
				{/if}
			</div>
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
