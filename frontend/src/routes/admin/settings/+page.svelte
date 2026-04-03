<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPut, apiPost } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let settings = $state({ site_name: '', site_description: '', logo_url: '', favicon_url: '', footer_text: '', social_links: '{}', custom_head_html: '' });
	let loading = $state(true);
	let saving = $state(false);
	let toast = $state(null);

	// Password change
	let currentPassword = $state('');
	let newPassword = $state('');
	let changingPassword = $state(false);

	onMount(async () => {
		try {
			const data = await apiGet('/admin/settings');
			settings = {
				...data,
				social_links: typeof data.social_links === 'string' ? data.social_links : JSON.stringify(data.social_links, null, 2)
			};
		} catch { /* empty */ }
		loading = false;
	});

	async function handleSave() {
		saving = true;
		let socialLinks;
		try {
			socialLinks = JSON.parse(settings.social_links);
		} catch {
			toast = { message: 'Social links JSON formatı hatalı', type: 'error' };
			saving = false;
			return;
		}

		const res = await apiPut('/admin/settings', {
			site_name: settings.site_name,
			site_description: settings.site_description,
			logo_url: settings.logo_url,
			favicon_url: settings.favicon_url,
			footer_text: settings.footer_text,
			social_links: socialLinks,
			custom_head_html: settings.custom_head_html
		});

		if (res.ok) {
			toast = { message: 'Ayarlar kaydedildi', type: 'success' };
		} else {
			toast = { message: 'Kaydetme başarısız', type: 'error' };
		}
		saving = false;
	}

	async function handleChangePassword() {
		if (!currentPassword || !newPassword) return;
		if (newPassword.length < 8) {
			toast = { message: 'Yeni şifre en az 8 karakter olmalıdır', type: 'error' };
			return;
		}
		changingPassword = true;
		const res = await apiPost('/admin/auth/change-password', {
			current_password: currentPassword,
			new_password: newPassword
		});
		if (res.ok) {
			toast = { message: 'Şifre değiştirildi', type: 'success' };
			currentPassword = '';
			newPassword = '';
		} else {
			const data = await res.json().catch(() => ({}));
			toast = { message: data.error || 'Şifre değiştirme başarısız', type: 'error' };
		}
		changingPassword = false;
	}
</script>

<svelte:head><title>Ayarlar - Admin</title></svelte:head>

<h1 class="page-heading">Site Ayarları</h1>

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else}
	<div class="settings-grid">
		<div class="card">
			<h2 class="card-title">Genel Ayarlar</h2>
			<div class="form-group">
				<label class="label">Site Adı</label>
				<input type="text" bind:value={settings.site_name} class="input" />
			</div>
			<div class="form-group">
				<label class="label">Site Açıklaması</label>
				<textarea bind:value={settings.site_description} class="textarea" rows="2"></textarea>
			</div>
			<div class="form-group">
				<label class="label">Logo URL</label>
				<input type="text" bind:value={settings.logo_url} class="input" placeholder="/uploads/logo.png" />
			</div>
			<div class="form-group">
				<label class="label">Favicon URL</label>
				<input type="text" bind:value={settings.favicon_url} class="input" />
			</div>
			<div class="form-group">
				<label class="label">Footer Metni</label>
				<textarea bind:value={settings.footer_text} class="textarea" rows="2"></textarea>
			</div>
			<div class="form-group">
				<label class="label">Sosyal Medya Linkleri (JSON)</label>
				<textarea bind:value={settings.social_links} class="textarea" rows="4"
						  style="font-family: var(--font-mono); font-size: 0.8125rem;"></textarea>
			</div>
			<div class="form-group">
				<label class="label">Özel Head HTML</label>
				<textarea bind:value={settings.custom_head_html} class="textarea" rows="3"
						  style="font-family: var(--font-mono); font-size: 0.8125rem;"
						  placeholder="<meta> veya <script> tagları"></textarea>
			</div>
			<button class="btn btn-primary" onclick={handleSave} disabled={saving}>
				{saving ? 'Kaydediliyor...' : 'Kaydet'}
			</button>
		</div>

		<div class="card">
			<h2 class="card-title">Şifre Değiştir</h2>
			<div class="form-group">
				<label class="label">Mevcut Şifre</label>
				<input type="password" bind:value={currentPassword} class="input" autocomplete="current-password" />
			</div>
			<div class="form-group">
				<label class="label">Yeni Şifre</label>
				<input type="password" bind:value={newPassword} class="input" autocomplete="new-password" />
			</div>
			<button class="btn btn-primary" onclick={handleChangePassword} disabled={changingPassword}>
				{changingPassword ? 'Değiştiriliyor...' : 'Şifreyi Değiştir'}
			</button>
		</div>
	</div>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; margin-bottom: 24px; }
	.card-title { font-size: 1rem; font-weight: 600; margin-bottom: 16px; }
	.settings-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }
	@media (max-width: 768px) { .settings-grid { grid-template-columns: 1fr; } }
</style>
