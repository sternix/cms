<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { apiGet, apiPut, apiPost } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let pageData = $state(null);
	let revisions = $state([]);
	let showRevisions = $state(false);
	let title = $state('');
	let slug = $state('');
	let content = $state('');
	let excerpt = $state('');
	let metaTitle = $state('');
	let metaDescription = $state('');
	let tags = $state('');
	let isVisible = $state(true);
	let saving = $state(false);
	let loading = $state(true);
	let toast = $state(null);
	let error = $state('');
	let previewMode = $state(false);

	const pageId = $page.params.id;

	onMount(async () => {
		if (pageId === 'new') { loading = false; return; }
		try {
			const data = await apiGet(`/admin/pages/${pageId}`);
			pageData = data;
			title = data.title;
			slug = data.slug;
			content = data.content;
			excerpt = data.excerpt;
			metaTitle = data.meta_title;
			metaDescription = data.meta_description;
			tags = data.tags.join(', ');
			isVisible = data.is_visible;
		} catch {
			error = 'Sayfa yüklenemedi.';
		}
		loading = false;
	});

	async function handleSave() {
		if (!title.trim()) { error = 'Başlık gereklidir.'; return; }
		error = '';
		saving = true;

		const res = await apiPut(`/admin/pages/${pageId}`, {
			title: title.trim(),
			slug: slug.trim(),
			content,
			excerpt: excerpt.trim(),
			meta_title: metaTitle.trim(),
			meta_description: metaDescription.trim(),
			tags: tags.trim() ? tags.split(',').map(t => t.trim()).filter(Boolean) : [],
			is_visible: isVisible
		});

		if (res.ok) {
			toast = { message: 'Sayfa güncellendi', type: 'success' };
		} else {
			const data = await res.json().catch(() => ({}));
			error = data.error || 'Güncelleme başarısız.';
		}
		saving = false;
	}

	async function loadRevisions() {
		showRevisions = !showRevisions;
		if (showRevisions && revisions.length === 0) {
			try {
				revisions = await apiGet(`/admin/pages/${pageId}/revisions`);
			} catch { /* empty */ }
		}
	}

	async function restoreRevision(revId) {
		if (!confirm('Bu revizyonu geri yüklemek istediğinize emin misiniz?')) return;
		try {
			const res = await apiPost(`/admin/pages/${pageId}/revisions/${revId}/restore`, {});
			if (res.ok) {
				const data = await res.json();
				title = data.title;
				content = data.content;
				excerpt = data.excerpt;
				metaTitle = data.meta_title;
				metaDescription = data.meta_description;
				tags = data.tags.join(', ');
				toast = { message: 'Revizyon geri yüklendi', type: 'success' };
				revisions = await apiGet(`/admin/pages/${pageId}/revisions`);
			}
		} catch { error = 'Geri yükleme başarısız.'; }
	}
</script>

<svelte:head>
	<title>{title || 'Sayfa Düzenle'} - Admin</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">{title || 'Sayfa Düzenle'}</h1>
	<div class="flex gap-2">
		<button class="btn btn-ghost btn-sm" onclick={() => previewMode = !previewMode}>
			{previewMode ? '✏ Düzenle' : '👁 Önizleme'}
		</button>
		<button class="btn btn-secondary btn-sm" onclick={loadRevisions}>
			{showRevisions ? 'Revizyonları Gizle' : '↺ Revizyonlar'}
		</button>
		<a href="/admin/pages" class="btn btn-ghost">← Geri</a>
	</div>
</div>

{#if loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else}
	{#if error}
		<div class="alert-error mb-3">{error}</div>
	{/if}

	{#if showRevisions && revisions.length > 0}
		<div class="card mb-4">
			<h3 style="margin-bottom:12px;">Revizyonlar</h3>
			<div class="revisions-list">
				{#each revisions as rev}
					<div class="revision-item">
						<span>Rev #{rev.revision_number}</span>
						<span class="text-muted text-sm">
							{new Date(rev.created_at).toLocaleString('tr-TR')}
						</span>
						<span class="text-sm">{rev.title}</span>
						<button class="btn btn-secondary btn-sm" onclick={() => restoreRevision(rev.id)}>
							Geri Yükle
						</button>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<div class="editor-layout">
		<div class="editor-main">
			{#if previewMode}
				<div class="card content">
					<h1>{title}</h1>
					{@html content}
				</div>
			{:else}
				<div class="form-group">
					<label for="title" class="label">Başlık *</label>
					<input id="title" type="text" bind:value={title} class="input" />
				</div>
				<div class="form-group">
					<label for="slug" class="label">Slug</label>
					<input id="slug" type="text" bind:value={slug} class="input" />
				</div>
				<div class="form-group">
					<label for="content" class="label">İçerik</label>
					<div class="editor-toolbar">
						<button type="button" class="tbtn" onclick={() => content += '<h2></h2>'}>H2</button>
						<button type="button" class="tbtn" onclick={() => content += '<h3></h3>'}>H3</button>
						<button type="button" class="tbtn" onclick={() => content += '<p></p>'}>P</button>
						<button type="button" class="tbtn" onclick={() => content += '<strong></strong>'}>B</button>
						<button type="button" class="tbtn" onclick={() => content += '<em></em>'}>I</button>
						<button type="button" class="tbtn" onclick={() => content += '<a href=""></a>'}>🔗</button>
						<button type="button" class="tbtn" onclick={() => content += '<img src="" alt="" />'}>🖼</button>
						<button type="button" class="tbtn" onclick={() => content += '<ul><li></li></ul>'}>☰</button>
						<button type="button" class="tbtn" onclick={() => content += '<blockquote></blockquote>'}>❝</button>
						<button type="button" class="tbtn" onclick={() => {
							const url = prompt('YouTube URL:');
							if (url) {
								const match = url.match(/(?:youtu\.be\/|youtube\.com\/(?:watch\?v=|embed\/))([^&?/]+)/);
								if (match) content += `<iframe width="560" height="315" src="https://www.youtube.com/embed/${match[1]}" frameborder="0" allowfullscreen></iframe>`;
							}
						}}>▶</button>
					</div>
					<textarea id="content" bind:value={content} class="textarea content-editor" rows="20"></textarea>
				</div>
			{/if}
		</div>

		<div class="editor-sidebar">
			<div class="card">
				<h3 class="card-title">Yayın Ayarları</h3>
				<div class="form-group">
					<label class="label">
						<input type="checkbox" bind:checked={isVisible} /> Görünür
					</label>
				</div>
				<div class="form-group">
					<label for="excerpt" class="label">Özet</label>
					<textarea id="excerpt" bind:value={excerpt} class="textarea" rows="3"></textarea>
				</div>
				<div class="form-group">
					<label for="tags" class="label">Etiketler</label>
					<input id="tags" type="text" bind:value={tags} class="input" placeholder="virgülle ayırın" />
				</div>
				<div class="form-group">
					<label for="metaTitle" class="label">Meta Başlık</label>
					<input id="metaTitle" type="text" bind:value={metaTitle} class="input" />
				</div>
				<div class="form-group">
					<label for="metaDesc" class="label">Meta Açıklama</label>
					<textarea id="metaDesc" bind:value={metaDescription} class="textarea" rows="2"></textarea>
				</div>
				<button class="btn btn-primary w-full" onclick={handleSave} disabled={saving}>
					{saving ? 'Kaydediliyor...' : 'Kaydet'}
				</button>
			</div>
		</div>
	</div>
{/if}

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.editor-layout { display: grid; grid-template-columns: 1fr 320px; gap: 24px; }
	.card-title { font-size: 1rem; font-weight: 600; margin-bottom: 16px; }
	.editor-toolbar {
		display: flex; flex-wrap: wrap; gap: 4px; padding: 8px;
		background: var(--color-bg-secondary); border: 1px solid var(--color-border);
		border-bottom: none; border-radius: var(--radius-md) var(--radius-md) 0 0;
	}
	.tbtn {
		padding: 4px 10px; border: 1px solid var(--color-border); border-radius: var(--radius-sm);
		background: var(--color-surface); color: var(--color-text); cursor: pointer;
		font-size: 0.8125rem; font-family: inherit;
	}
	.tbtn:hover { background: var(--color-surface-hover); }
	.content-editor {
		border-radius: 0 0 var(--radius-md) var(--radius-md);
		font-family: var(--font-mono); font-size: 0.875rem; min-height: 400px;
	}
	.alert-error {
		background: #fee2e2; color: var(--color-danger); padding: 10px 14px;
		border-radius: var(--radius-md); font-size: 0.875rem;
	}
	[data-theme='dark'] .alert-error { background: #7f1d1d; color: #fecaca; }
	.revisions-list { display: flex; flex-direction: column; gap: 8px; }
	.revision-item {
		display: flex; align-items: center; gap: 12px; padding: 8px 12px;
		border: 1px solid var(--color-border); border-radius: var(--radius-md);
	}
	.revision-item span:nth-child(3) { flex: 1; }

	@media (max-width: 768px) { .editor-layout { grid-template-columns: 1fr; } }
</style>
