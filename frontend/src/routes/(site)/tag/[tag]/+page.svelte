<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { stripTags } from '$lib/utils/sanitize.js';

	let tag = $state('');
	let pages = $state([]);
	let pagination = $state({ page: 1, totalPages: 1, total: 0 });
	let loading = $state(true);

	async function load(p = 1) {
		loading = true;
		try {
			const res = await fetch(`/api/tags/${encodeURIComponent(tag)}?page=${p}&per_page=12`);
			if (res.ok) {
				const data = await res.json();
				pages = data.data;
				pagination = { page: data.page, totalPages: data.total_pages, total: data.total };
			}
		} catch { /* empty */ }
		loading = false;
	}

	onMount(() => {
		tag = decodeURIComponent($page.params.tag);
		load();
	});

	function changePage(p) {
		load(p);
	}
</script>

<svelte:head>
	<title>Etiket: {tag}</title>
</svelte:head>

<div class="container">
	<h1 class="tag-heading">
		<span class="tag-label">Etiket:</span> {tag}
	</h1>

	{#if loading}
		<div class="loading-page"><div class="spinner"></div></div>
	{:else if pages.length === 0}
		<p class="text-center text-muted" style="padding: 40px 0;">Bu etiket için sayfa bulunamadı.</p>
	{:else}
		<p class="text-muted text-sm mb-3">{pagination.total} sayfa bulundu</p>
		<div class="pages-list">
			{#each pages as item}
				<a href="/page/{item.slug}" class="page-item card">
					<h2>{item.title}</h2>
					<p class="text-muted">{item.excerpt || stripTags(item.content).slice(0, 150)}</p>
				</a>
			{/each}
		</div>
		<Pagination page={pagination.page} totalPages={pagination.totalPages} onchange={changePage} />
	{/if}
</div>

<style>
	.tag-heading {
		font-size: 1.75rem;
		margin-bottom: 20px;
	}
	.tag-label {
		color: var(--color-text-secondary);
		font-weight: 400;
	}
	.pages-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.page-item {
		color: inherit;
		text-decoration: none;
	}
	.page-item h2 {
		font-size: 1.125rem;
		margin-bottom: 4px;
	}
</style>
