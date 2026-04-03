<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { stripTags } from '$lib/utils/sanitize.js';

	let results = $state([]);
	let query = $state('');
	let pagination = $state({ page: 1, totalPages: 1, total: 0 });
	let loading = $state(true);

	async function search(q, p = 1) {
		loading = true;
		try {
			const res = await fetch(`/api/search?q=${encodeURIComponent(q)}&page=${p}&per_page=12`);
			if (res.ok) {
				const data = await res.json();
				results = data.data;
				pagination = { page: data.page, totalPages: data.total_pages, total: data.total };
			}
		} catch { /* empty */ }
		loading = false;
	}

	onMount(() => {
		const params = new URLSearchParams($page.url.search);
		query = params.get('q') || '';
		if (query) search(query);
		else loading = false;
	});

	function handleSubmit(e) {
		e.preventDefault();
		if (query.trim()) {
			search(query.trim());
			history.replaceState(null, '', `/search?q=${encodeURIComponent(query.trim())}`);
		}
	}

	function changePage(p) {
		search(query, p);
	}
</script>

<svelte:head>
	<title>Arama{query ? `: ${query}` : ''}</title>
</svelte:head>

<div class="container">
	<h1 class="search-heading">Arama</h1>

	<form onsubmit={handleSubmit} class="search-form">
		<input type="search" bind:value={query} placeholder="Aranacak kelime..." class="input" />
		<button type="submit" class="btn btn-primary">Ara</button>
	</form>

	{#if loading}
		<div class="loading-page"><div class="spinner"></div></div>
	{:else if results.length === 0 && query}
		<p class="no-results">"{query}" için sonuç bulunamadı.</p>
	{:else if results.length > 0}
		<p class="result-count">{pagination.total} sonuç bulundu</p>
		<div class="results-list">
			{#each results as item}
				<a href="/page/{item.slug}" class="result-item card">
					<h2 class="result-title">{item.title}</h2>
					<p class="result-excerpt">{item.excerpt || stripTags(item.content).slice(0, 200)}</p>
					{#if item.tags.length > 0}
						<div class="flex gap-1 mt-1">
							{#each item.tags as tag}
								<span class="badge">{tag}</span>
							{/each}
						</div>
					{/if}
				</a>
			{/each}
		</div>
		<Pagination page={pagination.page} totalPages={pagination.totalPages} onchange={changePage} />
	{/if}
</div>

<style>
	.search-heading {
		font-size: 1.75rem;
		margin-bottom: 16px;
	}
	.search-form {
		display: flex;
		gap: 8px;
		margin-bottom: 24px;
	}
	.search-form .input { flex: 1; }
	.no-results {
		text-align: center;
		color: var(--color-text-secondary);
		padding: 40px 0;
	}
	.result-count {
		color: var(--color-text-secondary);
		font-size: 0.875rem;
		margin-bottom: 16px;
	}
	.results-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.result-item {
		color: inherit;
		text-decoration: none;
	}
	.result-title {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 4px;
	}
	.result-excerpt {
		color: var(--color-text-secondary);
		font-size: 0.9375rem;
	}
</style>
