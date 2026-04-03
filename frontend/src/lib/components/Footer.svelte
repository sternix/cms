<script>
	import { onMount } from 'svelte';

	let settings = $state({ site_name: 'CMS', footer_text: '', social_links: {} });
	let year = new Date().getFullYear();

	onMount(async () => {
		try {
			const res = await fetch('/api/site-settings');
			if (res.ok) settings = await res.json();
		} catch { /* defaults */ }
	});
</script>

<footer class="site-footer">
	<div class="container footer-inner">
		<div class="footer-info">
			<p class="footer-brand">{settings.site_name}</p>
			{#if settings.footer_text}
				<p class="footer-text">{settings.footer_text}</p>
			{/if}
		</div>
		<div class="footer-links">
			{#if settings.social_links}
				{#each Object.entries(settings.social_links) as [name, url]}
					<a href={url} target="_blank" rel="noopener noreferrer" class="social-link">
						{name}
					</a>
				{/each}
			{/if}
		</div>
		<p class="footer-copy">&copy; {year} {settings.site_name}. Tüm hakları saklıdır.</p>
	</div>
</footer>

<style>
	.site-footer {
		margin-top: auto;
		border-top: 1px solid var(--color-border);
		background: var(--color-bg-secondary);
		padding: 32px 0;
	}
	.footer-inner {
		display: flex;
		flex-direction: column;
		gap: 16px;
		align-items: center;
		text-align: center;
	}
	.footer-brand {
		font-weight: 600;
		font-size: 1.125rem;
		color: var(--color-text);
	}
	.footer-text {
		color: var(--color-text-secondary);
		font-size: 0.875rem;
	}
	.footer-links {
		display: flex;
		gap: 16px;
		flex-wrap: wrap;
		justify-content: center;
	}
	.social-link {
		color: var(--color-text-secondary);
		font-size: 0.875rem;
		text-transform: capitalize;
	}
	.social-link:hover { color: var(--color-primary); }
	.footer-copy {
		color: var(--color-text-muted);
		font-size: 0.8125rem;
	}
</style>
