<script lang="ts">
	import { ui } from '../stores.svelte';

	// Auto-hide notifications after a delay
	$effect(() => {
		if (ui.notification) {
			const timer = setTimeout(() => {
				ui.notification = null;
			}, 5000);

			return () => clearTimeout(timer);
		}
	});

	function closeNotification() {
		ui.notification = null;
	}
</script>

{#if ui.notification}
	<div class="fixed right-4 bottom-4 z-50 max-w-sm overflow-hidden rounded-lg shadow-lg">
		<div
			class="bg-white p-4 dark:bg-gray-800 {ui.notification.type === 'error'
				? 'border-l-4 border-red-500'
				: ui.notification.type === 'success'
					? 'border-l-4 border-green-500'
					: 'border-l-4 border-blue-500'}"
		>
			<div class="flex items-center">
				{#if ui.notification.type === 'error'}
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
							<path
								fill-rule="evenodd"
								d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
								clip-rule="evenodd"
							/>
						</svg>
					</div>
				{:else if ui.notification.type === 'success'}
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
							<path
								fill-rule="evenodd"
								d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
								clip-rule="evenodd"
							/>
						</svg>
					</div>
				{:else}
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
							<path
								fill-rule="evenodd"
								d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
								clip-rule="evenodd"
							/>
						</svg>
					</div>
				{/if}

				<div class="ml-3">
					<div class="text-sm font-medium text-gray-900 dark:text-white">
						{ui.notification.title ||
							(ui.notification.type === 'error'
								? 'Error'
								: ui.notification.type === 'success'
									? 'Success'
									: 'Info')}
					</div>
					<div class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						{ui.notification.message}
					</div>
				</div>

				<div class="ml-4 flex flex-shrink-0">
					<!-- svelte-ignore a11y_consider_explicit_label -->
					<button
						onclick={closeNotification}
						class="inline-flex text-gray-400 hover:text-gray-500 focus:text-gray-500 focus:outline-none"
					>
						<svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
							<path
								fill-rule="evenodd"
								d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
								clip-rule="evenodd"
							/>
						</svg>
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
