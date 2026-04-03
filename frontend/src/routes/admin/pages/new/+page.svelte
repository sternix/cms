<script>
	import { goto } from '$app/navigation';
	import { apiPost } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let title = $state('');
	let slug = $state('');
	let content = $state('');
	let excerpt = $state('');
	let metaTitle = $state('');
	let metaDescription = $state('');
	let tags = $state('');
	let isVisible = $state(true);
	let saving = $state(false);
	let toast = $state(null);
	let error = $state('');

	async function handleSave() {
		if (!title.trim()) { error = 'Başlık gereklidir.'; return; }
		error = '';
		saving = true;

		const res = await apiPost('/admin/pages', {
			title: title.trim(),
			slug: slug.trim() || undefined,
			content,
			excerpt: excerpt.trim() || undefined,
			meta_title: metaTitle.trim() || undefined,
			meta_description: metaDescription.trim() || undefined,
			tags: tags.trim() ? tags.split(',').map(t => t.trim()).filter(Boolean) : [],
			is_visible: isVisible
		});

		if (res.ok) {
			const data = await res.json();
			toast = { message: 'Sayfa oluşturuldu', type: 'success' };
			setTimeout(() => goto(`/admin/pages/${data.id}`), 500);
		} else {
			const data = await res.json().catch(() => ({}));
			error = data.error || 'Kaydetme başarısız.';
		}
		saving = false;
	}
</script>

<svelte:head>
	<title>Yeni Sayfa - Admin</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Yeni Sayfa</h1>
	<a href="/admin/pages" class="btn btn-ghost">← Geri</a>
</div>

{#if error}
	<div class="alert-error mb-3">{error}</div>
{/if}

<div class="editor-layout">
	<div class="editor-main">
		<div class="form-group">
			<label for="title" class="label">Başlık *</label>
			<input id="title" type="text" bind:value={title} class="input" placeholder="Sayfa başlığı" />
		</div>

		<div class="form-group">
			<label for="slug" class="label">Slug</label>
			<input id="slug" type="text" bind:value={slug} class="input" placeholder="otomatik oluşturulur" />
		</div>

		<div class="form-group">
			<label for="content" class="label">İçerik</label>
			<div class="editor-toolbar">
				<button type="button" class="tbtn" onclick={() => content += '<h2></h2>'} title="Başlık 2">H2</button>
				<button type="button" class="tbtn" onclick={() => content += '<h3></h3>'} title="Başlık 3">H3</button>
				<button type="button" class="tbtn" onclick={() => content += '<p></p>'} title="Paragraf">P</button>
				<button type="button" class="tbtn" onclick={() => content += '<strong></strong>'} title="Kalın">B</button>
				<button type="button" class="tbtn" onclick={() => content += '<em></em>'} title="İtalik">I</button>
				<button type="button" class="tbtn" onclick={() => content += '<a href=""></a>'} title="Link">🔗</button>
				<button type="button" class="tbtn" onclick={() => content += '<img src="" alt="" />'} title="Resim">🖼</button>
				<button type="button" class="tbtn" onclick={() => content += '<ul><li></li></ul>'} title="Liste">☰</button>
				<button type="button" class="tbtn" onclick={() => content += '<blockquote></blockquote>'} title="Alıntı">❝</button>
				<button type="button" class="tbtn" onclick={() => {
					const url = prompt('YouTube URL:');
					if (url) {
						const match = url.match(/(?:youtu\.be\/|youtube\.com\/(?:watch\?v=|embed\/))([^&?/]+)/);
						if (match) content += `<iframe width="560" height="315" src="https://www.youtube.com/embed/${match[1]}" frameborder="0" allowfullscreen></iframe>`;
					}
				}} title="Video">▶</button>
			</div>
			<textarea id="content" bind:value={content} class="textarea content-editor" rows="20"
					  placeholder="HTML içerik..."></textarea>
		</div>
	</div>

	<div class="editor-sidebar">
		<div class="card">
			<h3 class="card-title">Yayın Ayarları</h3>

			<div class="form-group">
				<label class="label">
					<input type="checkbox" bind:checked={isVisible} />
					Görünür
				</label>
			</div>

			<div class="form-group">
				<label for="excerpt" class="label">Özet</label>
				<textarea id="excerpt" bind:value={excerpt} class="textarea" rows="3"
						  placeholder="Kısa açıklama"></textarea>
			</div>

			<div class="form-group">
				<label for="tags" class="label">Etiketler</label>
				<input id="tags" type="text" bind:value={tags} class="input"
					   placeholder="virgülle ayırın" />
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
				{saving ? 'Kaydediliyor...' : 'Oluştur'}
			</button>
		</div>
	</div>
</div>

{#if toast}
	<Toast message={toast.message} type={toast.type} onclose={() => toast = null} />
{/if}

<style>
	.page-heading { font-size: 1.5rem; font-weight: 700; }
	.editor-layout {
		display: grid;
		grid-template-columns: 1fr 320px;
		gap: 24px;
	}
	.card-title {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 16px;
	}
	.editor-toolbar {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		padding: 8px;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-bottom: none;
		border-radius: var(--radius-md) var(--radius-md) 0 0;
	}
	.tbtn {
		padding: 4px 10px;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
		font-size: 0.8125rem;
		font-family: inherit;
	}
	.tbtn:hover { background: var(--color-surface-hover); }
	.content-editor {
		border-radius: 0 0 var(--radius-md) var(--radius-md);
		font-family: var(--font-mono);
		font-size: 0.875rem;
		min-height: 400px;
	}
	.alert-error {
		background: #fee2e2;
		color: var(--color-danger);
		padding: 10px 14px;
		border-radius: var(--radius-md);
		font-size: 0.875rem;
	}
	[data-theme='dark'] .alert-error { background: #7f1d1d; color: #fecaca; }

	@media (max-width: 768px) {
		.editor-layout { grid-template-columns: 1fr; }
	}
</style>
