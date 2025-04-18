<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    let name = $state("");
    let greetMsg = $state("");

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsg = await invoke("greet", { name });
    }
</script>

<main class="flex flex-col items-center justify-center pt-[10vh] text-center m-0">
    <h1 class="text-center">Welcome to Tauri + Svelte</h1>

    <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

    <form class="flex justify-center" onsubmit={greet}>
        <input
            id="greet-input"
            placeholder="Enter a name..."
            bind:value={name}
            class="mr-2 rounded-lg border border-transparent px-5 py-3 text-base font-medium shadow-md bg-white dark:bg-opacity-60 dark:bg-black dark:text-white transition-colors duration-250 outline-none hover:border-blue-600 focus:border-blue-600"
        />
        <button
            type="submit"
            class="rounded-lg border border-transparent px-5 py-3 text-base font-medium shadow-md bg-white dark:bg-opacity-60 dark:bg-black dark:text-white cursor-pointer transition-colors duration-250 outline-none hover:border-blue-600 active:border-blue-600 active:bg-stone-200 dark:active:bg-opacity-40"
        >
            Greet
        </button>
    </form>
    <p>{greetMsg}</p>
</main>
