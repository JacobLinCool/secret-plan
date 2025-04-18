<script lang="ts">
	import { ui } from '../stores.svelte';
</script>

{#if ui.isLoading}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-white/80 backdrop-blur-sm dark:bg-gray-900/80"
	>
		<div class="rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
			<div class="flex flex-col items-center">
				<div
					class="h-12 w-12 animate-spin rounded-full border-4 border-t-blue-600 border-r-blue-600 border-b-gray-200 border-l-gray-200"
				></div>
				<p class="mt-4 text-sm text-gray-600 dark:text-gray-400">
					{ui.loadingMessage || 'Loading...'}
				</p>
			</div>
		</div>
	</div>
{/if}
