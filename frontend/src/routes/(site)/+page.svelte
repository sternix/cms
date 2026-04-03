<script>
	import Slider from '$lib/components/Slider.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { onMount } from 'svelte';
	import { stripTags } from '$lib/utils/sanitize.js';

	let pages = $state([]);
	let tags = $state([]);
	let pagination = $state({ page: 1, totalPages: 1, total: 0 });
	let loading = $state(true);

	async function loadPages(page = 1) {
		loading = true;
		try {
			const res = await fetch(`/api/pages?page=${page}&per_page=12`);
			if (res.ok) {
				const data = await res.json();
				pages = data.data;
				pagination = { page: data.page, totalPages: data.total_pages, total: data.total };
			}
		} catch { /* empty */ }
		loading = false;
	}

	onMount(async () => {
		await loadPages();
		try {
			const res = await fetch('/api/tags');
			if (res.ok) tags = await res.json();
		} catch { /* empty */ }
	});

	function changePage(p) {
		loadPages(p);
		window.scrollTo({ top: 0, behavior: 'smooth' });
	}
</script>

<svelte:head>
	<title>Ana Sayfa</title>
</svelte:head>

<div class="container">
	<Slider />

	{#if tags.length > 0}
		<section class="tags-section">
			<div class="tags-list">
				{#each tags.slice(0, 15) as tag}
					<a href="/tag/{encodeURIComponent(tag.name)}" class="tag-chip">
						{tag.name}
						<span class="tag-count">{tag.count}</span>
					</a>
				{/each}
			</div>
		</section>
	{/if}

	{#if loading}
		<div class="loading-page"><div class="spinner"></div></div>
	{:else if pages.length === 0}
		<div class="empty-state">
			<p class="empty-icon">&#x1F4C4;</p>
			<p>Henüz yayınlanmış sayfa bulunmuyor.</p>
		</div>
	{:else}
		<section class="pages-grid">
			{#each pages as page}
				<a href="/page/{page.slug}" class="page-card card">
					<h2 class="page-card-title">{page.title}</h2>
					<p class="page-card-excerpt">
						{page.excerpt || stripTags(page.content).slice(0, 150) + '...'}
					</p>
					<div class="page-card-meta">
						<time class="text-sm text-muted">
							{new Date(page.created_at).toLocaleDateString('tr-TR')}
						</time>
						{#if page.tags.length > 0}
							<div class="page-card-tags">
								{#each page.tags.slice(0, 3) as tag}
									<span class="badge">{tag}</span>
								{/each}
							</div>
						{/if}
					</div>
					{#if page.is_pinned}
						<span class="pin-badge" title="Sabitlenmiş">&#x1F4CC;</span>
					{/if}
				</a>
			{/each}
		</section>

		<Pagination page={pagination.page} totalPages={pagination.totalPages} onchange={changePage} />
	{/if}
</div>

<style>
	.tags-section {
		margin-bottom: 24px;
	}
	.tags-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.tag-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: 9999px;
		font-size: 0.8125rem;
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}
	.tag-chip:hover {
		border-color: var(--color-primary);
		color: var(--color-primary);
		background: var(--color-primary-light);
	}
	.tag-count {
		font-size: 0.75rem;
		opacity: 0.6;
	}
	.pages-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 20px;
	}
	.page-card {
		display: flex;
		flex-direction: column;
		color: inherit;
		position: relative;
		text-decoration: none;
	}
	.page-card:hover { color: inherit; }
	.page-card-title {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 8px;
		color: var(--color-text);
	}
	.page-card-excerpt {
		color: var(--color-text-secondary);
		font-size: 0.9375rem;
		line-height: 1.5;
		flex: 1;
		margin-bottom: 12px;
	}
	.page-card-meta {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		flex-wrap: wrap;
	}
	.page-card-tags {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.pin-badge {
		position: absolute;
		top: 12px;
		right: 12px;
		font-size: 1rem;
	}
	.empty-state {
		text-align: center;
		padding: 60px 20px;
		color: var(--color-text-secondary);
	}
	.empty-icon {
		font-size: 3rem;
		margin-bottom: 12px;
	}

	@media (max-width: 640px) {
		.pages-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
