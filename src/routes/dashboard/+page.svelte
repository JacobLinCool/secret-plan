<script lang="ts">
	import {
		credentialStore,
		vault,
		lockVault,
		loadCredentials,
		filters,
		initDarkMode,
		toggleDarkMode,
		ui
	} from '$lib/stores.svelte';
	import { goto } from '$app/navigation';
	import LoadingOverlay from '$lib/components/LoadingOverlay.svelte';
	import ErrorNotification from '$lib/components/ErrorNotification.svelte';
	import CredentialCard from '$lib/components/CredentialCard.svelte';
	import AddCredentialForm from '$lib/components/AddCredentialForm.svelte';
	import { onMount } from 'svelte';

	// UI state
	let showAddForm = $state(false);
	let showSidebar = $state(false);
	let uniqueTags = $state<string[]>([]);

	// Initialize dark mode on component mount
	// Load credentials on component mount
	onMount(async () => {
		initDarkMode();
		await loadCredentials();
	});

	// Check if vault is locked, if so redirect to login
	$effect(() => {
		if (vault.isLocked) {
			goto('/');
		}
	});

	// Extract unique tags from credentials
	$effect(() => {
		const tagSet = new Set<string>();

		credentialStore.items.forEach((credential) => {
			for (const tag of credential.tags) {
				tagSet.add(tag);
			}
		});

		uniqueTags = Array.from(tagSet).sort();
	});

	// Logout function
	async function handleLogout() {
		await lockVault();
		goto('/');
	}

	// Filter functions
	function clearFilters() {
		filters.searchTerm = '';
		filters.selectedTag = null;
		filters.minStrength = null;
		filters.breachFilter = null;
		loadCredentials();
	}

	function selectTag(tag: string | null) {
		filters.selectedTag = filters.selectedTag === tag ? null : tag;
		loadCredentials();
	}

	function selectBreachFilter(value: number | null) {
		filters.breachFilter = filters.breachFilter === value ? null : value;
		loadCredentials();
	}

	function setMinStrength(value: number | null) {
		filters.minStrength = filters.minStrength === value ? null : value;
		loadCredentials();
	}

	async function handleSearch(e: Event) {
		e.preventDefault();
		await loadCredentials();
	}
</script>

<LoadingOverlay />
<ErrorNotification />

<div class="flex min-h-screen dark:bg-gray-900">
	<!-- Sidebar (hidden on mobile) -->
	<div
		class="hidden w-60 flex-shrink-0 flex-col bg-gradient-to-b from-blue-800 to-blue-600 p-4 md:flex dark:from-gray-800 dark:to-gray-700"
	>
		<div class="mb-8 flex items-center justify-center">
			<h1 class="text-xl font-bold text-white">Secret Plan</h1>
		</div>

		<div class="mb-4">
			<h2
				class="mb-2 text-sm font-medium tracking-wider text-blue-200 uppercase dark:text-gray-400"
			>
				Tags
			</h2>
			<div class="space-y-1">
				<button
					onclick={() => (filters.selectedTag = null)}
					class="{filters.selectedTag === null
						? 'bg-blue-700 font-medium dark:bg-gray-600'
						: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
				>
					All Credentials
				</button>

				{#each uniqueTags as tag}
					<button
						onclick={() => selectTag(tag)}
						class="{filters.selectedTag === tag
							? 'bg-blue-700 font-medium dark:bg-gray-600'
							: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full truncate rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
					>
						{tag}
					</button>
				{/each}
			</div>
		</div>

		<div class="mb-4">
			<h2
				class="mb-2 text-sm font-medium tracking-wider text-blue-200 uppercase dark:text-gray-400"
			>
				Security
			</h2>
			<div class="space-y-1">
				<button
					onclick={() => selectBreachFilter(1)}
					class="{filters.breachFilter === 1
						? 'bg-blue-700 font-medium dark:bg-gray-600'
						: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
				>
					Safe Passwords
				</button>
				<button
					onclick={() => selectBreachFilter(2)}
					class="{filters.breachFilter === 2
						? 'bg-blue-700 font-medium dark:bg-gray-600'
						: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
				>
					Compromised
				</button>
				<button
					onclick={() => selectBreachFilter(0)}
					class="{filters.breachFilter === 0
						? 'bg-blue-700 font-medium dark:bg-gray-600'
						: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
				>
					Unchecked
				</button>
			</div>
		</div>

		<div class="mt-auto">
			<button
				onclick={toggleDarkMode}
				class="mb-2 flex w-full items-center rounded px-3 py-2 text-sm text-white hover:bg-blue-700/50 dark:hover:bg-gray-600/50"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mr-2 h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					{#if ui.darkMode}
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
						/>
					{:else}
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
						/>
					{/if}
				</svg>
				{ui.darkMode ? 'Light Mode' : 'Dark Mode'}
			</button>

			<button
				onclick={handleLogout}
				class="flex w-full items-center rounded px-3 py-2 text-sm text-white hover:bg-blue-700/50 dark:hover:bg-gray-600/50"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mr-2 h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
					/>
				</svg>
				Lock Vault
			</button>
		</div>
	</div>

	<!-- Mobile sidebar toggle -->
	<div class="fixed bottom-4 left-4 z-10 md:hidden">
		<button
			onclick={() => (showSidebar = !showSidebar)}
			class="rounded-full bg-blue-600 p-3 text-white shadow-lg"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				{#if showSidebar}
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M6 18L18 6M6 6l12 12"
					/>
				{:else}
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 6h16M4 12h16M4 18h16"
					/>
				{/if}
			</svg>
		</button>
	</div>

	<!-- Mobile sidebar -->
	{#if showSidebar}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm md:hidden"
			onclick={() => (showSidebar = false)}
		>
			<div
				class="h-full w-64 bg-gradient-to-b from-blue-800 to-blue-600 p-4 dark:from-gray-800 dark:to-gray-700"
			>
				<div class="mb-8 flex items-center justify-between">
					<h1 class="text-xl font-bold text-white">Secret Plan</h1>
					<!-- svelte-ignore a11y_consider_explicit_label -->
					<button onclick={() => (showSidebar = false)}>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-6 w-6 text-white"
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

				<div class="mb-4">
					<h2
						class="mb-2 text-sm font-medium tracking-wider text-blue-200 uppercase dark:text-gray-400"
					>
						Tags
					</h2>
					<div class="space-y-1">
						<button
							onclick={() => {
								filters.selectedTag = null;
								loadCredentials();
								showSidebar = false;
							}}
							class="{filters.selectedTag === null
								? 'bg-blue-700 font-medium dark:bg-gray-600'
								: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
						>
							All Credentials
						</button>

						{#each uniqueTags as tag}
							<button
								onclick={() => {
									selectTag(tag);
									showSidebar = false;
								}}
								class="{filters.selectedTag === tag
									? 'bg-blue-700 font-medium dark:bg-gray-600'
									: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full truncate rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
							>
								{tag}
							</button>
						{/each}
					</div>
				</div>

				<div class="mb-4">
					<h2
						class="mb-2 text-sm font-medium tracking-wider text-blue-200 uppercase dark:text-gray-400"
					>
						Security
					</h2>
					<div class="space-y-1">
						<button
							onclick={() => {
								selectBreachFilter(1);
								showSidebar = false;
							}}
							class="{filters.breachFilter === 1
								? 'bg-blue-700 font-medium dark:bg-gray-600'
								: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
						>
							Safe Passwords
						</button>
						<button
							onclick={() => {
								selectBreachFilter(2);
								showSidebar = false;
							}}
							class="{filters.breachFilter === 2
								? 'bg-blue-700 font-medium dark:bg-gray-600'
								: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
						>
							Compromised
						</button>
						<button
							onclick={() => {
								selectBreachFilter(0);
								showSidebar = false;
							}}
							class="{filters.breachFilter === 0
								? 'bg-blue-700 font-medium dark:bg-gray-600'
								: 'hover:bg-blue-700/50 dark:hover:bg-gray-600/50'} w-full rounded px-3 py-2 text-left text-sm text-white transition-colors duration-200"
						>
							Unchecked
						</button>
					</div>
				</div>

				<div class="mt-auto">
					<button
						onclick={() => {
							toggleDarkMode();
							showSidebar = false;
						}}
						class="mb-2 flex w-full items-center rounded px-3 py-2 text-sm text-white hover:bg-blue-700/50 dark:hover:bg-gray-600/50"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="mr-2 h-5 w-5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							{#if ui.darkMode}
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
								/>
							{:else}
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
								/>
							{/if}
						</svg>
						{ui.darkMode ? 'Light Mode' : 'Dark Mode'}
					</button>

					<button
						onclick={handleLogout}
						class="flex w-full items-center rounded px-3 py-2 text-sm text-white hover:bg-blue-700/50 dark:hover:bg-gray-600/50"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="mr-2 h-5 w-5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
							/>
						</svg>
						Lock Vault
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Main content -->
	<div class="flex-1 bg-gray-50 dark:bg-gray-900">
		<!-- Header -->
		<header class="bg-white p-4 shadow dark:bg-gray-800">
			<div class="mx-auto flex items-center justify-between">
				<div class="md:hidden">
					<h1 class="text-xl font-bold text-gray-900 dark:text-white">Secret Plan</h1>
				</div>

				<div class="flex flex-1 items-center md:mx-4">
					<form onsubmit={handleSearch} class="w-full">
						<div class="relative">
							<input
								type="text"
								bind:value={filters.searchTerm}
								placeholder="Search sites, usernames..."
								class="w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 pr-10 text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
							/>
							<!-- svelte-ignore a11y_consider_explicit_label -->
							<button
								type="submit"
								class="absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-1.5 text-gray-500 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 focus:outline-none dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
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
										d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
									/>
								</svg>
							</button>
						</div>
					</form>
				</div>

				<div>
					<button
						onclick={() => {
							showAddForm = true;
						}}
						class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-700 dark:hover:bg-blue-800 dark:focus:ring-blue-800"
					>
						Add New
					</button>
				</div>
			</div>
		</header>

		<main class="p-4 md:p-6">
			<!-- Active filters -->
			{#if filters.searchTerm || filters.selectedTag || filters.minStrength !== null || filters.breachFilter !== null}
				<div class="mb-4 flex flex-wrap items-center gap-2">
					<span class="text-sm text-gray-600 dark:text-gray-400">Active filters:</span>

					{#if filters.searchTerm}
						<div
							class="flex items-center rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>
							Search: "{filters.searchTerm}"
							<button
								onclick={() => {
									filters.searchTerm = '';
									loadCredentials();
								}}
								class="ml-1 text-blue-600 hover:text-blue-800 dark:text-blue-300"
							>
								×
							</button>
						</div>
					{/if}

					{#if filters.selectedTag}
						<div
							class="flex items-center rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>
							Tag: {filters.selectedTag}
							<button
								onclick={() => {
									filters.selectedTag = null;
									loadCredentials();
								}}
								class="ml-1 text-blue-600 hover:text-blue-800 dark:text-blue-300"
							>
								×
							</button>
						</div>
					{/if}

					{#if filters.minStrength !== null}
						<div
							class="flex items-center rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>
							Min Strength: {filters.minStrength}%
							<button
								onclick={() => {
									filters.minStrength = null;
									loadCredentials();
								}}
								class="ml-1 text-blue-600 hover:text-blue-800 dark:text-blue-300"
							>
								×
							</button>
						</div>
					{/if}

					{#if filters.breachFilter !== null}
						<div
							class="flex items-center rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
						>
							Breach Status: {filters.breachFilter === 0
								? 'Unknown'
								: filters.breachFilter === 1
									? 'Safe'
									: 'Compromised'}
							<button
								onclick={() => {
									filters.breachFilter = null;
									loadCredentials();
								}}
								class="ml-1 text-blue-600 hover:text-blue-800 dark:text-blue-300"
							>
								×
							</button>
						</div>
					{/if}

					<button
						onclick={clearFilters}
						class="ml-auto rounded bg-gray-200 px-2 py-1 text-xs text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
					>
						Clear All
					</button>
				</div>
			{/if}

			<!-- Strength filter (visible on mobile) -->
			<div class="mb-4 md:hidden">
				<div class="mb-2 text-sm font-medium text-gray-600 dark:text-gray-400">
					Filter by strength:
				</div>
				<div class="flex flex-wrap gap-1">
					<button
						onclick={() => setMinStrength(80)}
						class="rounded-full px-3 py-1 text-xs font-medium {filters.minStrength === 80
							? 'bg-green-500 text-white'
							: 'bg-green-100 text-green-800 hover:bg-green-200 dark:bg-green-900/20 dark:text-green-300'}"
					>
						Strong
					</button>
					<button
						onclick={() => setMinStrength(60)}
						class="rounded-full px-3 py-1 text-xs font-medium {filters.minStrength === 60
							? 'bg-yellow-500 text-white'
							: 'bg-yellow-100 text-yellow-800 hover:bg-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-300'}"
					>
						Good+
					</button>
					<button
						onclick={() => setMinStrength(40)}
						class="rounded-full px-3 py-1 text-xs font-medium {filters.minStrength === 40
							? 'bg-orange-500 text-white'
							: 'bg-orange-100 text-orange-800 hover:bg-orange-200 dark:bg-orange-900/20 dark:text-orange-300'}"
					>
						Fair+
					</button>
				</div>
			</div>

			<!-- Credentials grid -->
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each credentialStore.items as credential (credential.uuid)}
					<CredentialCard {credential} />
				{/each}

				{#if credentialStore.items.length === 0}
					<div
						class="col-span-full flex flex-col items-center justify-center rounded-lg border border-gray-200 bg-white p-8 text-center dark:border-gray-700 dark:bg-gray-800"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="mb-4 h-12 w-12 text-gray-400"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
							/>
						</svg>
						<h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-white">
							No credentials found
						</h3>
						<p class="mb-4 text-gray-600 dark:text-gray-400">
							{#if filters.searchTerm || filters.selectedTag || filters.minStrength !== null || filters.breachFilter !== null}
								No credentials match your current filters. Try modifying your search.
							{:else}
								You haven't added any credentials yet. Start by adding your first password.
							{/if}
						</p>
						{#if filters.searchTerm || filters.selectedTag || filters.minStrength !== null || filters.breachFilter !== null}
							<button
								onclick={clearFilters}
								class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
							>
								Clear Filters
							</button>
						{:else}
							<button
								onclick={() => {
									showAddForm = true;
								}}
								class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:ring-4 focus:ring-blue-300 focus:outline-none dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800"
							>
								Add Your First Credential
							</button>
						{/if}
					</div>
				{/if}
			</div>
		</main>
	</div>

	<!-- Add Credential Modal -->
	{#if showAddForm}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
		>
			<AddCredentialForm
				onClose={() => {
					showAddForm = false;
				}}
			/>
		</div>
	{/if}
</div>
