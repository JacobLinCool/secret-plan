<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	let name = $state('');
	let greetMsg = $state('');

	async function greet(event: Event) {
		event.preventDefault();
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		greetMsg = await invoke('greet', { name });
	}
</script>

<main class="m-0 flex flex-col items-center justify-center pt-[10vh] text-center">
	<h1 class="text-center">Welcome to Tauri + Svelte</h1>

	<p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

	<form class="flex justify-center" onsubmit={greet}>
		<input
			id="greet-input"
			placeholder="Enter a name..."
			bind:value={name}
			class="dark:bg-opacity-60 mr-2 rounded-lg border border-transparent bg-white px-5 py-3 text-base font-medium shadow-md transition-colors duration-250 outline-none hover:border-blue-600 focus:border-blue-600 dark:bg-black dark:text-white"
		/>
		<button
			type="submit"
			class="dark:bg-opacity-60 dark:active:bg-opacity-40 cursor-pointer rounded-lg border border-transparent bg-white px-5 py-3 text-base font-medium shadow-md transition-colors duration-250 outline-none hover:border-blue-600 active:border-blue-600 active:bg-stone-200 dark:bg-black dark:text-white"
		>
			Greet
		</button>
	</form>
	<p>{greetMsg}</p>
</main>
