<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	let pageData = $state(null);
	let loading = $state(true);
	let error = $state(null);

	onMount(async () => {
		const slug = $page.params.slug;
		try {
			const res = await fetch(`/api/pages/${encodeURIComponent(slug)}`);
			if (res.ok) {
				pageData = await res.json();
			} else if (res.status === 404) {
				error = 'Sayfa bulunamadı.';
			} else {
				error = 'Bir hata oluştu.';
			}
		} catch {
			error = 'Bağlantı hatası.';
		}
		loading = false;
	});
</script>

<svelte:head>
	{#if pageData}
		<title>{pageData.meta_title || pageData.title}</title>
		{#if pageData.meta_description}
			<meta name="description" content={pageData.meta_description} />
		{/if}
	{/if}
</svelte:head>

<div class="container">
	{#if loading}
		<div class="loading-page"><div class="spinner"></div></div>
	{:else if error}
		<div class="error-page">
			<h1>&#x26A0; Hata</h1>
			<p>{error}</p>
			<a href="/" class="btn btn-primary mt-3">Ana Sayfaya Dön</a>
		</div>
	{:else if pageData}
		<article class="page-detail">
			<header class="page-header">
				<h1 class="page-title">{pageData.title}</h1>
				<div class="page-meta">
					<time class="text-muted text-sm">
						{new Date(pageData.created_at).toLocaleDateString('tr-TR', { year: 'numeric', month: 'long', day: 'numeric' })}
					</time>
					{#if pageData.updated_at !== pageData.created_at}
						<span class="text-muted text-sm">
							(Güncellendi: {new Date(pageData.updated_at).toLocaleDateString('tr-TR')})
						</span>
					{/if}
				</div>
				{#if pageData.tags.length > 0}
					<div class="page-tags">
						{#each pageData.tags as tag}
							<a href="/tag/{encodeURIComponent(tag)}" class="badge">{tag}</a>
						{/each}
					</div>
				{/if}
			</header>

			<div class="content">
				{@html pageData.content}
			</div>
		</article>
	{/if}
</div>

<style>
	.page-detail {
		max-width: 800px;
		margin: 0 auto;
	}
	.page-header {
		margin-bottom: 32px;
		padding-bottom: 20px;
		border-bottom: 1px solid var(--color-border);
	}
	.page-title {
		font-size: 2.25rem;
		font-weight: 800;
		line-height: 1.2;
		margin-bottom: 12px;
	}
	.page-meta {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
		margin-bottom: 12px;
	}
	.page-tags {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.error-page {
		text-align: center;
		padding: 60px 20px;
	}
	.error-page h1 {
		font-size: 1.5rem;
		margin-bottom: 8px;
	}

	@media (max-width: 640px) {
		.page-title { font-size: 1.5rem; }
	}
</style>
