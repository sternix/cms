<script>
	import { page } from '$app/stores';
</script>

<div class="error-page">
	<div class="error-content">
		<h1 class="error-code">{$page.status}</h1>
		<p class="error-message">{$page.error?.message || 'Bir hata oluştu.'}</p>
		<a href="/" class="btn btn-primary btn-lg">Ana Sayfaya Dön</a>
	</div>
</div>

<style>
	.error-page {
		min-height: 80vh;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 20px;
	}
	.error-content { text-align: center; }
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
		margin-bottom: 24px;
	}
</style>
