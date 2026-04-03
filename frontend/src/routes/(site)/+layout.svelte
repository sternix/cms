<script>
	import Header from '$lib/components/Header.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(async () => {
		// Track page visit
		try {
			await fetch('/api/analytics/track', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					page_path: window.location.pathname,
					referrer: document.referrer || null
				})
			});
		} catch { /* silent */ }
	});
</script>

<div class="site-layout">
	<Header />
	<main class="site-main">
		{@render children()}
	</main>
	<Footer />
</div>

<style>
	.site-layout {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}
	.site-main {
		flex: 1;
		padding: 24px 0;
	}
</style>
