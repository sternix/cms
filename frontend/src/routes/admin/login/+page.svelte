<script>
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth.svelte.js';
	import { theme } from '$lib/stores/theme.svelte.js';
	import { onMount } from 'svelte';

	let username = $state('');
	let password = $state('');
	let captchaId = $state('');
	let captchaImage = $state('');
	let captchaAnswer = $state('');
	let error = $state('');
	let loading = $state(false);

	async function loadCaptcha() {
		try {
			const res = await fetch('/api/captcha');
			if (res.ok) {
				const data = await res.json();
				captchaId = data.id;
				captchaImage = data.image;
				captchaAnswer = '';
			}
		} catch {
			error = 'Captcha yüklenemedi.';
		}
	}

	onMount(() => {
		theme.init();
		loadCaptcha();
	});

	async function handleLogin(e) {
		e.preventDefault();
		error = '';

		if (!username.trim() || !password.trim()) {
			error = 'Kullanıcı adı ve şifre gereklidir.';
			return;
		}
		if (!captchaAnswer.trim()) {
			error = 'Captcha cevabı gereklidir.';
			return;
		}

		loading = true;
		try {
			const res = await fetch('/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					username: username.trim(),
					password,
					captcha_id: captchaId,
					captcha_answer: captchaAnswer.trim()
				})
			});

			if (res.ok) {
				const data = await res.json();
				auth.login(data.token, data.user);
				goto('/admin/dashboard');
			} else {
				const data = await res.json().catch(() => ({}));
				error = data.error || 'Giriş başarısız.';
				loadCaptcha();
			}
		} catch {
			error = 'Bağlantı hatası.';
		}
		loading = false;
	}
</script>

<svelte:head>
	<title>Admin Girişi</title>
</svelte:head>

<div class="login-page">
	<div class="login-card">
		<div class="login-header">
			<div class="login-logo">&#x2699;</div>
			<h1>Yönetim Paneli</h1>
			<p class="text-muted">Devam etmek için giriş yapın</p>
		</div>

		{#if error}
			<div class="alert-error">{error}</div>
		{/if}

		<form onsubmit={handleLogin}>
			<div class="form-group">
				<label for="username" class="label">Kullanıcı Adı</label>
				<input id="username" type="text" bind:value={username} class="input"
					   autocomplete="username" required />
			</div>

			<div class="form-group">
				<label for="password" class="label">Şifre</label>
				<input id="password" type="password" bind:value={password} class="input"
					   autocomplete="current-password" required />
			</div>

			<div class="form-group">
				<label class="label">Güvenlik Kodu</label>
				{#if captchaImage}
					<div class="captcha-row">
						<img src={captchaImage} alt="Captcha" class="captcha-img" />
						<button type="button" class="btn btn-ghost btn-sm" onclick={loadCaptcha}>&#x21BB;</button>
					</div>
				{/if}
				<input type="text" bind:value={captchaAnswer} class="input mt-1"
					   placeholder="Sonucu girin" required />
			</div>

			<button type="submit" class="btn btn-primary w-full btn-lg" disabled={loading}>
				{loading ? 'Giriş yapılıyor...' : 'Giriş Yap'}
			</button>
		</form>

		<div class="login-footer">
			<button class="btn btn-ghost btn-sm" onclick={() => theme.toggle()}>
				{theme.current === 'dark' ? '☀ Açık Tema' : '☾ Koyu Tema'}
			</button>
		</div>
	</div>
</div>

<style>
	.login-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-bg-secondary);
		padding: 20px;
	}
	.login-card {
		width: 100%;
		max-width: 420px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-xl);
		padding: 36px;
	}
	.login-header {
		text-align: center;
		margin-bottom: 28px;
	}
	.login-logo {
		font-size: 2.5rem;
		margin-bottom: 8px;
	}
	.login-header h1 {
		font-size: 1.5rem;
		font-weight: 700;
	}
	.alert-error {
		background: #fee2e2;
		color: var(--color-danger);
		padding: 10px 14px;
		border-radius: var(--radius-md);
		font-size: 0.875rem;
		margin-bottom: 16px;
	}
	[data-theme='dark'] .alert-error {
		background: #7f1d1d;
		color: #fecaca;
	}
	.captcha-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.captcha-img {
		height: 48px;
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}
	.login-footer {
		text-align: center;
		margin-top: 20px;
	}
</style>
