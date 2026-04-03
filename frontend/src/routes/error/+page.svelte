<script>
	import { page } from '$app/stores';
</script>

<svelte:head>
	<title>Hata</title>
</svelte:head>

<div class="error-page">
	<div class="error-content">
		<h1 class="error-code">{$page.status || 404}</h1>
		<p class="error-message">
			{#if $page.status === 404}
				Aradığınız sayfa bulunamadı.
			{:else if $page.status === 500}
				Sunucu hatası oluştu.
			{:else}
				Bir hata oluştu.
			{/if}
		</p>
		<a href="/" class="btn btn-primary btn-lg mt-4">Ana Sayfaya Dön</a>
	</div>
</div>

<style>
	.error-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 20px;
	}
	.error-content {
		text-align: center;
	}
	.error-code {
		font-size: 6rem;
		font-weight: 800;
		color: var(--color-primary);
		line-height: 1;
		margin-bottom: 8px;
	}
	.error-message {
		font-size: 1.25rem;
		color: var(--color-text-secondary);
	}
</style>
