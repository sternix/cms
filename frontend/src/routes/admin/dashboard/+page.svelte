<script>
	import { onMount } from 'svelte';
	import { apiGet } from '$lib/utils/api.js';

	let summary = $state(null);
	let topPages = $state([]);
	let dailyStats = $state([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			const [s, tp, ds] = await Promise.all([
				apiGet('/admin/analytics/summary'),
				apiGet('/admin/analytics/top-pages?limit=10'),
				apiGet('/admin/analytics/daily?days=14')
			]);
			summary = s;
			topPages = tp;
			dailyStats = ds;
		} catch { /* empty */ }
		loading = false;
	});
</script>

<svelte:head>
	<title>Gösterge Paneli - Admin</title>
</svelte:head>

<h1 class="page-heading">Gösterge Paneli</h1>

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else if summary}
	<div class="stats-grid">
		<div class="stat-card card">
			<div class="stat-value">{summary.total_visits.toLocaleString('tr-TR')}</div>
			<div class="stat-label">Toplam Ziyaret</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">{summary.unique_visitors.toLocaleString('tr-TR')}</div>
			<div class="stat-label">Tekil Ziyaretçi</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">{summary.visits_today.toLocaleString('tr-TR')}</div>
			<div class="stat-label">Bugünkü Ziyaret</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">{summary.visits_this_month.toLocaleString('tr-TR')}</div>
			<div class="stat-label">Bu Ay</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">{summary.total_pages}</div>
			<div class="stat-label">Toplam Sayfa</div>
		</div>
	</div>

	<div class="dashboard-grid">
		<div class="card">
			<h2 class="card-title">Son 14 Gün</h2>
			{#if dailyStats.length > 0}
				{@const maxVisits = Math.max(...dailyStats.map(d => d.visits), 1)}
				<div class="chart-bars">
					{#each dailyStats.slice().reverse() as day}
						<div class="chart-bar-item" title="{day.date}: {day.visits} ziyaret">
							<div class="chart-bar" style="height: {(day.visits / maxVisits) * 100}%"></div>
							<span class="chart-bar-label">{day.date.slice(8)}</span>
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-muted">Henüz veri yok.</p>
			{/if}
		</div>

		<div class="card">
			<h2 class="card-title">En Çok Ziyaret Edilen</h2>
			{#if topPages.length > 0}
				<div class="top-pages-list">
					{#each topPages as tp, i}
						<div class="top-page-item">
							<span class="top-page-rank">{i + 1}</span>
							<span class="top-page-path">{tp.page_path}</span>
							<span class="top-page-count">{tp.visits}</span>
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-muted">Henüz veri yok.</p>
			{/if}
		</div>
	</div>
{/if}

<style>
	.page-heading {
		font-size: 1.5rem;
		font-weight: 700;
		margin-bottom: 24px;
	}
	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 16px;
		margin-bottom: 24px;
	}
	.stat-card {
		text-align: center;
		padding: 20px;
	}
	.stat-value {
		font-size: 2rem;
		font-weight: 800;
		color: var(--color-primary);
		line-height: 1.2;
	}
	.stat-label {
		font-size: 0.8125rem;
		color: var(--color-text-secondary);
		margin-top: 4px;
	}
	.dashboard-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}
	.card-title {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 16px;
	}
	.chart-bars {
		display: flex;
		align-items: flex-end;
		gap: 4px;
		height: 160px;
	}
	.chart-bar-item {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		height: 100%;
		justify-content: flex-end;
	}
	.chart-bar {
		width: 100%;
		max-width: 32px;
		background: var(--color-primary);
		border-radius: var(--radius-sm) var(--radius-sm) 0 0;
		min-height: 2px;
		transition: height var(--transition-base);
	}
	.chart-bar-label {
		font-size: 0.625rem;
		color: var(--color-text-muted);
		margin-top: 4px;
	}
	.top-pages-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.top-page-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 0;
		border-bottom: 1px solid var(--color-border-light);
		font-size: 0.875rem;
	}
	.top-page-rank {
		width: 24px;
		text-align: center;
		font-weight: 600;
		color: var(--color-text-muted);
	}
	.top-page-path {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.top-page-count {
		font-weight: 600;
		color: var(--color-primary);
	}

	@media (max-width: 768px) {
		.dashboard-grid { grid-template-columns: 1fr; }
	}
</style>
