<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiPut, apiDelete } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let sliders = $state([]);
	let loading = $state(true);
	let toast = $state(null);
	let showForm = $state(false);
	let editId = $state(null);
	let form = $state({ title: '', description: '', image_url: '', link_url: '' });
	let draggedId = $state(null);

	async function load() {
		try { sliders = await apiGet('/admin/sliders'); } catch { /* empty */ }
		loading = false;
	}

	onMount(load);

	function openNew() {
		editId = null;
		form = { title: '', description: '', image_url: '', link_url: '' };
		showForm = true;
	}

	function openEdit(s) {
		editId = s.id;
		form = { title: s.title, description: s.description, image_url: s.image_url, link_url: s.link_url };
		showForm = true;
	}

	async function handleSave() {
		if (!form.title.trim() || !form.image_url.trim()) return;

		let res;
		if (editId) {
			res = await apiPut(`/admin/sliders/${editId}`, form);
		} else {
			res = await apiPost('/admin/sliders', form);
		}

		if (res.ok) {
			toast = { message: editId ? 'Slider güncellendi' : 'Slider oluşturuldu', type: 'success' };
			showForm = false;
			await load();
		}
	}

	async function deleteSlider(s) {
		if (!confirm(`"${s.title}" slider'ını silmek istediğinize emin misiniz?`)) return;
		const res = await apiDelete(`/admin/sliders/${s.id}`);
		if (res.ok) {
			sliders = sliders.filter(sl => sl.id !== s.id);
			toast = { message: 'Slider silindi', type: 'success' };
		}
	}

	async function toggleVis(s) {
		await apiPut(`/admin/sliders/${s.id}/visibility`, { value: !s.is_visible });
		s.is_visible = !s.is_visible;
		sliders = [...sliders];
	}

	async function togglePin(s) {
		await apiPut(`/admin/sliders/${s.id}/pin`, { value: !s.is_pinned });
		s.is_pinned = !s.is_pinned;
		sliders = [...sliders];
	}

	function onDragStart(e, s) { if (!s.is_pinned) draggedId = s.id; else e.preventDefault(); }
	function onDragOver(e, s) { if (!s.is_pinned) e.preventDefault(); }
	async function onDrop(e, target) {
		e.preventDefault();
		if (!draggedId || target.is_pinned) return;
		const di = sliders.findIndex(s => s.id === draggedId);
		const ti = sliders.findIndex(s => s.id === target.id);
		if (di === ti) return;
		const item = sliders.splice(di, 1)[0];
		sliders.splice(ti, 0, item);
		sliders = [...sliders];
		const items = sliders.filter(s => !s.is_pinned).map((s, i) => ({ id: s.id, sort_order: i }));
		await apiPut('/admin/sliders/reorder', { items });
		draggedId = null;
	}
</script>

<svelte:head><title>Slider Yönetimi - Admin</title></svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Slider Yönetimi</h1>
	<button class="btn btn-primary" onclick={openNew}>+ Yeni Slider</button>
</div>

{#if showForm}
	<div class="card mb-4">
		<h3 style="margin-bottom: 12px">{editId ? 'Slider Düzenle' : 'Yeni Slider'}</h3>
		<div class="form-row">
			<div class="form-group" style="flex:1">
				<label class="label">Başlık *</label>
				<input type="text" bind:value={form.title} class="input" />
			</div>
			<div class="form-group" style="flex:1">
				<label class="label">Resim URL *</label>
				<input type="text" bind:value={form.image_url} class="input" placeholder="/uploads/..." />
			</div>
		</div>
		<div class="form-row">
			<div class="form-group" style="flex:1">
				<label class="label">Açıklama</label>
				<input type="text" bind:value={form.description} class="input" />
			</div>
			<div class="form-group" style="flex:1">
				<label class="label">Link URL</label>
				<input type="text" bind:value={form.link_url} class="input" placeholder="/page/..." />
			</div>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-primary" onclick={handleSave}>Kaydet</button>
			<button class="btn btn-ghost" onclick={() => showForm = false}>İptal</button>
		</div>
	</div>
{/if}

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else if sliders.length === 0}
	<p class="text-muted text-center" style="padding:40px">Henüz slider yok.</p>
{:else}
	<div class="slider-list">
		{#each sliders as s (s.id)}
			<div class="slider-item card"
				 draggable={!s.is_pinned}
				 ondragstart={(e) => onDragStart(e, s)}
				 ondragover={(e) => onDragOver(e, s)}
				 ondrop={(e) => onDrop(e, s)}
				 class:pinned={s.is_pinned}>
				<div class="slider-thumb">
					{#if s.image_url}
						<img src={s.image_url} alt={s.title} />
					{:else}
						<div class="thumb-placeholder">⊡</div>
					{/if}
				</div>
				<div class="slider-info">
					<h3>{s.title}</h3>
					{#if s.description}<p class="text-muted text-sm">{s.description}</p>{/if}
					{#if s.link_url}<p class="text-sm">Link: {s.link_url}</p>{/if}
				</div>
				<div class="slider-actions">
					<span class="badge" class:badge-success={s.is_visible} class:badge-warning={!s.is_visible}>
						{s.is_visible ? 'Görünür' : 'Gizli'}
					</span>
					<button class="btn btn-ghost btn-sm" onclick={() => togglePin(s)}>{s.is_pinned ? '📌' : '○'}</button>
					<button class="btn btn-ghost btn-sm" onclick={() => toggleVis(s)}>{s.is_visible ? '👁' : '👁‍🗨'}</button>
					<button class="btn btn-secondary btn-sm" onclick={() => openEdit(s)}>Düzenle</button>
					<button class="btn btn-danger btn-sm" onclick={() => deleteSlider(s)}>Sil</button>
				</div>
			</div>
		{/each}
	</div>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.form-row { display: flex; gap: 16px; }
	.slider-list { display: flex; flex-direction: column; gap: 12px; }
	.slider-item { display: flex; align-items: center; gap: 16px; cursor: grab; }
	.slider-item.pinned { border-left: 3px solid var(--color-primary); }
	.slider-thumb { width: 120px; height: 68px; flex-shrink: 0; border-radius: var(--radius-md); overflow: hidden; background: var(--color-bg-secondary); }
	.slider-thumb img { width: 100%; height: 100%; object-fit: cover; }
	.thumb-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 1.5rem; color: var(--color-text-muted); }
	.slider-info { flex: 1; }
	.slider-info h3 { font-size: 1rem; font-weight: 600; }
	.slider-actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }

	@media (max-width: 640px) {
		.form-row { flex-direction: column; }
		.slider-item { flex-direction: column; align-items: stretch; }
		.slider-thumb { width: 100%; height: 120px; }
		.slider-actions { flex-wrap: wrap; }
	}
</style>
