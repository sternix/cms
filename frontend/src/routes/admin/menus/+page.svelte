<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiPut, apiDelete } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let menus = $state([]);
	let loading = $state(true);
	let toast = $state(null);
	let showForm = $state(false);
	let editId = $state(null);
	let form = $state({ label: '', url: '', parent_id: null, open_in_new_tab: false });
	let draggedId = $state(null);

	async function load() {
		try { menus = await apiGet('/admin/menus'); } catch { /* empty */ }
		loading = false;
	}

	onMount(load);

	function openNew() {
		editId = null;
		form = { label: '', url: '', parent_id: null, open_in_new_tab: false };
		showForm = true;
	}

	function openEdit(m) {
		editId = m.id;
		form = { label: m.label, url: m.url, parent_id: m.parent_id, open_in_new_tab: m.open_in_new_tab };
		showForm = true;
	}

	async function handleSave() {
		if (!form.label.trim()) return;
		let res;
		if (editId) {
			res = await apiPut(`/admin/menus/${editId}`, form);
		} else {
			res = await apiPost('/admin/menus', form);
		}
		if (res.ok) {
			toast = { message: editId ? 'Menü güncellendi' : 'Menü oluşturuldu', type: 'success' };
			showForm = false;
			await load();
		}
	}

	async function deleteMenu(m) {
		if (!confirm(`"${m.label}" menüsünü silmek istediğinize emin misiniz?`)) return;
		const res = await apiDelete(`/admin/menus/${m.id}`);
		if (res.ok) {
			menus = menus.filter(item => item.id !== m.id);
			toast = { message: 'Menü silindi', type: 'success' };
		}
	}

	function onDragStart(e, m) { draggedId = m.id; }
	function onDragOver(e) { e.preventDefault(); }
	async function onDrop(e, target) {
		e.preventDefault();
		if (!draggedId) return;
		const di = menus.findIndex(m => m.id === draggedId);
		const ti = menus.findIndex(m => m.id === target.id);
		if (di === ti) return;
		const item = menus.splice(di, 1)[0];
		menus.splice(ti, 0, item);
		menus = [...menus];
		const items = menus.map((m, i) => ({ id: m.id, sort_order: i }));
		await apiPut('/admin/menus/reorder', { items });
		draggedId = null;
	}
</script>

<svelte:head><title>Menü Yönetimi - Admin</title></svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Menü Yönetimi</h1>
	<button class="btn btn-primary" onclick={openNew}>+ Yeni Menü</button>
</div>

{#if showForm}
	<div class="card mb-4">
		<h3 style="margin-bottom: 12px">{editId ? 'Menü Düzenle' : 'Yeni Menü'}</h3>
		<div class="form-row">
			<div class="form-group" style="flex:1">
				<label class="label">Etiket *</label>
				<input type="text" bind:value={form.label} class="input" />
			</div>
			<div class="form-group" style="flex:1">
				<label class="label">URL</label>
				<input type="text" bind:value={form.url} class="input" placeholder="/page/..." />
			</div>
		</div>
		<div class="form-group">
			<label class="label">
				<input type="checkbox" bind:checked={form.open_in_new_tab} /> Yeni sekmede aç
			</label>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-primary" onclick={handleSave}>Kaydet</button>
			<button class="btn btn-ghost" onclick={() => showForm = false}>İptal</button>
		</div>
	</div>
{/if}

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else if menus.length === 0}
	<p class="text-muted text-center" style="padding:40px">Henüz menü yok.</p>
{:else}
	<div class="table-wrap">
		<table>
			<thead><tr><th>⇅</th><th>Etiket</th><th>URL</th><th>Yeni Sekme</th><th>Görünür</th><th>İşlemler</th></tr></thead>
			<tbody>
				{#each menus as m (m.id)}
					<tr draggable="true"
						ondragstart={(e) => onDragStart(e, m)}
						ondragover={onDragOver}
						ondrop={(e) => onDrop(e, m)}>
						<td style="cursor:grab; color:var(--color-text-muted)">⋮⋮</td>
						<td>{m.label}</td>
						<td class="text-muted text-sm">{m.url}</td>
						<td>{m.open_in_new_tab ? '✓' : '—'}</td>
						<td>
							<span class="badge" class:badge-success={m.is_visible} class:badge-warning={!m.is_visible}>
								{m.is_visible ? 'Evet' : 'Hayır'}
							</span>
						</td>
						<td>
							<div class="flex gap-1">
								<button class="btn btn-secondary btn-sm" onclick={() => openEdit(m)}>Düzenle</button>
								<button class="btn btn-danger btn-sm" onclick={() => deleteMenu(m)}>Sil</button>
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.form-row { display: flex; gap: 16px; }
	@media (max-width: 640px) { .form-row { flex-direction: column; } }
</style>
