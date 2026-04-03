<script>
	let { page = 1, totalPages = 1, onchange = () => {} } = $props();
</script>

{#if totalPages > 1}
	<nav class="pagination" aria-label="Sayfalama">
		<button
			class="btn btn-ghost btn-sm"
			disabled={page <= 1}
			onclick={() => onchange(page - 1)}
		>
			&#x2190; Önceki
		</button>

		<span class="page-info">{page} / {totalPages}</span>

		<button
			class="btn btn-ghost btn-sm"
			disabled={page >= totalPages}
			onclick={() => onchange(page + 1)}
		>
			Sonraki &#x2192;
		</button>
	</nav>
{/if}

<style>
	.pagination {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 16px;
		margin-top: 24px;
		padding: 16px 0;
	}
	.page-info {
		color: var(--color-text-secondary);
		font-size: 0.875rem;
	}
</style>
