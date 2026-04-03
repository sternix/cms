<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiGet, apiPost, apiPut, apiDelete } from '$lib/utils/api.js';
	import Pagination from '$lib/components/Pagination.svelte';
	import Toast from '$lib/components/Toast.svelte';

	let pages = $state([]);
	let pagination = $state({ page: 1, totalPages: 1, total: 0 });
	let loading = $state(true);
	let toast = $state(null);
	let draggedId = $state(null);

	async function load(p = 1) {
		loading = true;
		try {
			const data = await apiGet(`/admin/pages?page=${p}&per_page=50`);
			pages = data.data;
			pagination = { page: data.page, totalPages: data.total_pages, total: data.total };
		} catch { /* empty */ }
		loading = false;
	}

	onMount(() => load());

	async function toggleVisibility(page) {
		const res = await apiPut(`/admin/pages/${page.id}/visibility`, { value: !page.is_visible });
		if (res.ok) {
			page.is_visible = !page.is_visible;
			pages = [...pages];
			toast = { message: page.is_visible ? 'Sayfa görünür yapıldı' : 'Sayfa gizlendi', type: 'success' };
		}
	}

	async function togglePin(page) {
		const res = await apiPut(`/admin/pages/${page.id}/pin`, { value: !page.is_pinned });
		if (res.ok) {
			page.is_pinned = !page.is_pinned;
			pages = [...pages];
			toast = { message: page.is_pinned ? 'Sayfa sabitlendi' : 'Sabitleme kaldırıldı', type: 'success' };
		}
	}

	async function deletePage(page) {
		if (!confirm(`"${page.title}" sayfasını silmek istediğinize emin misiniz?`)) return;
		const res = await apiDelete(`/admin/pages/${page.id}`);
		if (res.ok) {
			pages = pages.filter(p => p.id !== page.id);
			toast = { message: 'Sayfa silindi', type: 'success' };
		}
	}

	// Drag & Drop
	function onDragStart(e, page) {
		if (page.is_pinned) { e.preventDefault(); return; }
		draggedId = page.id;
		e.dataTransfer.effectAllowed = 'move';
	}

	function onDragOver(e, page) {
		if (page.is_pinned) return;
		e.preventDefault();
		e.dataTransfer.dropEffect = 'move';
	}

	async function onDrop(e, targetPage) {
		e.preventDefault();
		if (!draggedId || targetPage.is_pinned) return;

		const draggedIndex = pages.findIndex(p => p.id === draggedId);
		const targetIndex = pages.findIndex(p => p.id === targetPage.id);

		if (draggedIndex === targetIndex) return;

		const item = pages.splice(draggedIndex, 1)[0];
		pages.splice(targetIndex, 0, item);
		pages = [...pages];

		const items = pages.filter(p => !p.is_pinned).map((p, i) => ({ id: p.id, sort_order: i }));
		await apiPut('/admin/pages/reorder', { items });
		toast = { message: 'Sıralama güncellendi', type: 'success' };
		draggedId = null;
	}
</script>

<svelte:head>
	<title>Sayfalar - Admin</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Sayfalar</h1>
	<a href="/admin/pages/new" class="btn btn-primary">+ Yeni Sayfa</a>
</div>

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else}
	<div class="table-wrap">
		<table>
			<thead>
				<tr>
					<th style="width:32px">⇅</th>
					<th>Başlık</th>
					<th>Slug</th>
					<th>Durum</th>
					<th>Sabit</th>
					<th>Tarih</th>
					<th style="width:200px">İşlemler</th>
				</tr>
			</thead>
			<tbody>
				{#each pages as page (page.id)}
					<tr draggable={!page.is_pinned}
						ondragstart={(e) => onDragStart(e, page)}
						ondragover={(e) => onDragOver(e, page)}
						ondrop={(e) => onDrop(e, page)}
						class:dragging={draggedId === page.id}
						class:pinned={page.is_pinned}>
						<td class="drag-handle">{page.is_pinned ? '📌' : '⋮⋮'}</td>
						<td>
							<a href="/admin/pages/{page.id}" class="page-title-link">{page.title}</a>
						</td>
						<td class="text-muted text-sm">/{page.slug}</td>
						<td>
							<span class="badge" class:badge-success={page.is_visible} class:badge-warning={!page.is_visible}>
								{page.is_visible ? 'Görünür' : 'Gizli'}
							</span>
						</td>
						<td>
							<button class="btn btn-ghost btn-sm" onclick={() => togglePin(page)}>
								{page.is_pinned ? '📌' : '○'}
							</button>
						</td>
						<td class="text-muted text-sm">
							{new Date(page.created_at).toLocaleDateString('tr-TR')}
						</td>
						<td>
							<div class="flex gap-1">
								<a href="/admin/pages/{page.id}" class="btn btn-secondary btn-sm">Düzenle</a>
								<button class="btn btn-ghost btn-sm" onclick={() => toggleVisibility(page)}>
									{page.is_visible ? '👁' : '👁‍🗨'}
								</button>
								<button class="btn btn-danger btn-sm" onclick={() => deletePage(page)}>Sil</button>
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<Pagination page={pagination.page} totalPages={pagination.totalPages} onchange={load} />
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.drag-handle {
		cursor: grab;
		user-select: none;
		text-align: center;
		color: var(--color-text-muted);
	}
	tr.dragging { opacity: 0.5; }
	tr.pinned { background: var(--color-primary-light); }
	.page-title-link {
		font-weight: 500;
		color: var(--color-text);
	}
	.page-title-link:hover { color: var(--color-primary); }
</style>
