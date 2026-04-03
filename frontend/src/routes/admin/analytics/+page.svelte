<script>
	import { onMount } from 'svelte';
	import { apiGet } from '$lib/utils/api.js';

	let summary = $state(null);
	let dailyStats = $state([]);
	let monthlyStats = $state([]);
	let topPages = $state([]);
	let loading = $state(true);
	let activeTab = $state('daily');

	onMount(async () => {
		try {
			const [s, ds, ms, tp] = await Promise.all([
				apiGet('/admin/analytics/summary'),
				apiGet('/admin/analytics/daily?days=30'),
				apiGet('/admin/analytics/monthly?months=12'),
				apiGet('/admin/analytics/top-pages?limit=20')
			]);
			summary = s;
			dailyStats = ds;
			monthlyStats = ms;
			topPages = tp;
		} catch { /* empty */ }
		loading = false;
	});
</script>

<svelte:head><title>Analitik - Admin</title></svelte:head>

<h1 class="page-heading">Analitik</h1>

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
			<div class="stat-label">Bugün</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">{summary.visits_this_month.toLocaleString('tr-TR')}</div>
			<div class="stat-label">Bu Ay</div>
		</div>
	</div>

	<div class="tabs mb-3">
		<button class="tab" class:active={activeTab === 'daily'} onclick={() => activeTab = 'daily'}>Günlük</button>
		<button class="tab" class:active={activeTab === 'monthly'} onclick={() => activeTab = 'monthly'}>Aylık</button>
		<button class="tab" class:active={activeTab === 'top'} onclick={() => activeTab = 'top'}>En Çok Ziyaret</button>
	</div>

	{#if activeTab === 'daily'}
		<div class="card">
			<h2 class="card-title">Son 30 Gün</h2>
			{#if dailyStats.length > 0}
				{@const maxV = Math.max(...dailyStats.map(d => d.visits), 1)}
				<div class="chart-bars">
					{#each dailyStats.slice().reverse() as day}
						<div class="chart-bar-item" title="{day.date}: {day.visits} ziyaret, {day.unique_visitors} tekil">
							<div class="chart-bar" style="height: {(day.visits / maxV) * 100}%"></div>
							<span class="chart-bar-label">{day.date.slice(5)}</span>
						</div>
					{/each}
				</div>
				<div class="table-wrap mt-3">
					<table>
						<thead><tr><th>Tarih</th><th>Ziyaret</th><th>Tekil</th></tr></thead>
						<tbody>
							{#each dailyStats as d}
								<tr><td>{d.date}</td><td>{d.visits}</td><td>{d.unique_visitors}</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="text-muted">Veri yok.</p>
			{/if}
		</div>
	{:else if activeTab === 'monthly'}
		<div class="card">
			<h2 class="card-title">Son 12 Ay</h2>
			{#if monthlyStats.length > 0}
				{@const maxV = Math.max(...monthlyStats.map(d => d.visits), 1)}
				<div class="chart-bars">
					{#each monthlyStats.slice().reverse() as month}
						<div class="chart-bar-item" title="{month.month}: {month.visits} ziyaret">
							<div class="chart-bar" style="height: {(month.visits / maxV) * 100}%"></div>
							<span class="chart-bar-label">{month.month.slice(5)}</span>
						</div>
					{/each}
				</div>
				<div class="table-wrap mt-3">
					<table>
						<thead><tr><th>Ay</th><th>Ziyaret</th><th>Tekil</th></tr></thead>
						<tbody>
							{#each monthlyStats as m}
								<tr><td>{m.month}</td><td>{m.visits}</td><td>{m.unique_visitors}</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="text-muted">Veri yok.</p>
			{/if}
		</div>
	{:else}
		<div class="card">
			<h2 class="card-title">En Çok Ziyaret Edilen Sayfalar</h2>
			<div class="table-wrap">
				<table>
					<thead><tr><th>#</th><th>Sayfa</th><th>Ziyaret</th></tr></thead>
					<tbody>
						{#each topPages as tp, i}
							<tr><td>{i+1}</td><td>{tp.page_path}</td><td>{tp.visits}</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; margin-bottom: 24px; }
	.stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 16px; margin-bottom: 24px; }
	.stat-card { text-align: center; padding: 20px; }
	.stat-value { font-size: 2rem; font-weight: 800; color: var(--color-primary); }
	.stat-label { font-size: 0.8125rem; color: var(--color-text-secondary); margin-top: 4px; }
	.card-title { font-size: 1rem; font-weight: 600; margin-bottom: 16px; }
	.tabs { display: flex; gap: 4px; }
	.tab {
		padding: 8px 16px; border: 1px solid var(--color-border); border-radius: var(--radius-md);
		background: var(--color-bg-secondary); color: var(--color-text-secondary); cursor: pointer;
		font-family: inherit; font-size: 0.875rem;
	}
	.tab.active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
	.chart-bars { display: flex; align-items: flex-end; gap: 3px; height: 180px; }
	.chart-bar-item { flex: 1; display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: flex-end; }
	.chart-bar { width: 100%; max-width: 28px; background: var(--color-primary); border-radius: var(--radius-sm) var(--radius-sm) 0 0; min-height: 2px; }
	.chart-bar-label { font-size: 0.5625rem; color: var(--color-text-muted); margin-top: 4px; white-space: nowrap; }
</style>
