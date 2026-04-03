<script>
	import { onMount } from 'svelte';
	import { apiGet, apiDelete, api } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let mediaList = $state([]);
	let loading = $state(true);
	let uploading = $state(false);
	let toast = $state(null);
	let selected = $state(null);
	let cropForm = $state({ width: '', height: '', crop_x: '', crop_y: '', crop_width: '', crop_height: '' });
	let transforming = $state(false);

	async function load() {
		try { mediaList = await apiGet('/admin/media'); } catch { /* empty */ }
		loading = false;
	}

	onMount(load);

	async function handleUpload(e) {
		const files = e.target.files;
		if (!files.length) return;
		uploading = true;

		for (const file of files) {
			const formData = new FormData();
			formData.append('file', file);

			const res = await api('/admin/media/upload', {
				method: 'POST',
				body: formData
			});

			if (res.ok) {
				const media = await res.json();
				mediaList = [media, ...mediaList];
				toast = { message: `${file.name} yüklendi`, type: 'success' };
			} else {
				const data = await res.json().catch(() => ({}));
				toast = { message: data.error || 'Yükleme başarısız', type: 'error' };
			}
		}
		uploading = false;
		e.target.value = '';
	}

	async function deleteMedia(m) {
		if (!confirm('Bu medyayı silmek istediğinize emin misiniz?')) return;
		const res = await apiDelete(`/admin/media/${m.id}`);
		if (res.ok) {
			mediaList = mediaList.filter(item => item.id !== m.id);
			if (selected?.id === m.id) selected = null;
			toast = { message: 'Medya silindi', type: 'success' };
		}
	}

	function selectMedia(m) {
		selected = selected?.id === m.id ? null : m;
		if (selected) {
			cropForm = { width: m.width || '', height: m.height || '', crop_x: '', crop_y: '', crop_width: '', crop_height: '' };
		}
	}

	function copyUrl(url) {
		navigator.clipboard.writeText(url);
		toast = { message: 'URL kopyalandı', type: 'success' };
	}

	async function handleTransform() {
		if (!selected) return;
		transforming = true;
		const body = {};
		if (cropForm.width) body.width = parseInt(cropForm.width);
		if (cropForm.height) body.height = parseInt(cropForm.height);
		if (cropForm.crop_x) body.crop_x = parseInt(cropForm.crop_x);
		if (cropForm.crop_y) body.crop_y = parseInt(cropForm.crop_y);
		if (cropForm.crop_width) body.crop_width = parseInt(cropForm.crop_width);
		if (cropForm.crop_height) body.crop_height = parseInt(cropForm.crop_height);

		const res = await api(`/admin/media/${selected.id}/transform`, {
			method: 'POST',
			body: JSON.stringify(body),
			headers: { 'Content-Type': 'application/json' }
		});

		if (res.ok) {
			const updated = await res.json();
			mediaList = mediaList.map(m => m.id === updated.id ? updated : m);
			selected = updated;
			toast = { message: 'Resim dönüştürüldü', type: 'success' };
		} else {
			toast = { message: 'Dönüştürme başarısız', type: 'error' };
		}
		transforming = false;
	}
</script>

<svelte:head><title>Medya Yönetimi - Admin</title></svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Medya Yönetimi</h1>
	<label class="btn btn-primary">
		{uploading ? 'Yükleniyor...' : '+ Yükle'}
		<input type="file" accept="image/*" multiple onchange={handleUpload} hidden />
	</label>
</div>

{#if selected}
	<div class="card mb-4 media-detail">
		<div class="media-detail-inner">
			<div class="media-preview">
				<img src={selected.url} alt={selected.original_name} />
			</div>
			<div class="media-info">
				<h3>{selected.original_name}</h3>
				<p class="text-sm text-muted">{selected.mime_type} &middot; {(selected.size_bytes / 1024).toFixed(1)} KB</p>
				{#if selected.width && selected.height}
					<p class="text-sm text-muted">{selected.width} &times; {selected.height}px</p>
				{/if}
				<div class="flex gap-2 mt-2">
					<button class="btn btn-secondary btn-sm" onclick={() => copyUrl(selected.url)}>URL Kopyala</button>
					<button class="btn btn-danger btn-sm" onclick={() => deleteMedia(selected)}>Sil</button>
				</div>

				<h4 class="mt-3" style="font-size:0.875rem;font-weight:600;">Resize / Crop</h4>
				<div class="transform-form">
					<div class="form-row-sm">
						<input type="number" bind:value={cropForm.width} class="input" placeholder="Genişlik" />
						<input type="number" bind:value={cropForm.height} class="input" placeholder="Yükseklik" />
					</div>
					<div class="form-row-sm">
						<input type="number" bind:value={cropForm.crop_x} class="input" placeholder="Crop X" />
						<input type="number" bind:value={cropForm.crop_y} class="input" placeholder="Crop Y" />
					</div>
					<div class="form-row-sm">
						<input type="number" bind:value={cropForm.crop_width} class="input" placeholder="Crop W" />
						<input type="number" bind:value={cropForm.crop_height} class="input" placeholder="Crop H" />
					</div>
					<button class="btn btn-primary btn-sm" onclick={handleTransform} disabled={transforming}>
						{transforming ? 'İşleniyor...' : 'Uygula'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else if mediaList.length === 0}
	<p class="text-muted text-center" style="padding:40px">Henüz medya yok. Yüklemek için butona tıklayın.</p>
{:else}
	<div class="media-grid">
		{#each mediaList as m (m.id)}
			<button class="media-card" class:selected={selected?.id === m.id} onclick={() => selectMedia(m)}>
				<div class="media-thumb">
					<img src={m.url} alt={m.original_name} loading="lazy" />
				</div>
				<p class="media-name">{m.original_name}</p>
			</button>
		{/each}
	</div>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.media-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
		gap: 12px;
	}
	.media-card {
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		border-radius: var(--radius-md);
		overflow: hidden;
		cursor: pointer;
		transition: border-color var(--transition-fast);
		text-align: left;
		padding: 0;
		font-family: inherit;
		color: inherit;
	}
	.media-card:hover { border-color: var(--color-primary); }
	.media-card.selected { border-color: var(--color-primary); box-shadow: 0 0 0 2px var(--color-primary-light); }
	.media-thumb {
		aspect-ratio: 1;
		overflow: hidden;
		background: var(--color-bg-secondary);
	}
	.media-thumb img { width: 100%; height: 100%; object-fit: cover; }
	.media-name {
		padding: 6px 8px;
		font-size: 0.75rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		color: var(--color-text-secondary);
	}
	.media-detail-inner {
		display: flex;
		gap: 20px;
	}
	.media-preview {
		width: 300px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		overflow: hidden;
		background: var(--color-bg-secondary);
	}
	.media-preview img { width: 100%; }
	.media-info { flex: 1; }
	.media-info h3 { font-size: 1rem; font-weight: 600; margin-bottom: 4px; }
	.transform-form { margin-top: 8px; }
	.form-row-sm { display: flex; gap: 8px; margin-bottom: 8px; }
	.form-row-sm .input { padding: 6px 10px; font-size: 0.8125rem; }

	@media (max-width: 640px) {
		.media-detail-inner { flex-direction: column; }
		.media-preview { width: 100%; }
	}
</style>
