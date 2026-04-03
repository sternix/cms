<script>
	import { onMount } from 'svelte';

	let { message = '', type = 'success', duration = 3000, onclose = () => {} } = $props();
	let visible = $state(true);

	onMount(() => {
		const timer = setTimeout(() => {
			visible = false;
			onclose();
		}, duration);
		return () => clearTimeout(timer);
	});
</script>

{#if visible}
	<div class="toast toast-{type}" role="alert">
		{message}
	</div>
{/if}
