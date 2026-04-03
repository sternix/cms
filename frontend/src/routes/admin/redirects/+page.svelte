<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiDelete } from '$lib/utils/api.js';
	import Toast from '$lib/components/Toast.svelte';

	let redirects = $state([]);
	let loading = $state(true);
	let toast = $state(null);
	let showForm = $state(false);
	let form = $state({ from_path: '', to_path: '', status_code: 301 });

	async function load() {
		try { redirects = await apiGet('/admin/redirects'); } catch { /* empty */ }
		loading = false;
	}

	onMount(load);

	async function handleSave() {
		if (!form.from_path.trim() || !form.to_path.trim()) return;
		const res = await apiPost('/admin/redirects', form);
		if (res.ok) {
			const data = await res.json();
			redirects = [data, ...redirects];
			showForm = false;
			toast = { message: 'Yönlendirme oluşturuldu', type: 'success' };
		} else {
			const data = await res.json().catch(() => ({}));
			toast = { message: data.error || 'Oluşturma başarısız', type: 'error' };
		}
	}

	async function deleteRedirect(r) {
		if (!confirm('Bu yönlendirmeyi silmek istediğinize emin misiniz?')) return;
		const res = await apiDelete(`/admin/redirects/${r.id}`);
		if (res.ok) {
			redirects = redirects.filter(item => item.id !== r.id);
			toast = { message: 'Yönlendirme silindi', type: 'success' };
		}
	}
</script>

<svelte:head><title>Yönlendirmeler - Admin</title></svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="page-heading">Yönlendirmeler</h1>
	<button class="btn btn-primary" onclick={() => { form = { from_path: '', to_path: '', status_code: 301 }; showForm = true; }}>
		+ Yeni Yönlendirme
	</button>
</div>

{#if showForm}
	<div class="card mb-4">
		<h3 style="margin-bottom:12px">Yeni Yönlendirme</h3>
		<div class="form-row">
			<div class="form-group" style="flex:1">
				<label class="label">Kaynak Yol</label>
				<input type="text" bind:value={form.from_path} class="input" placeholder="/eski-sayfa" />
			</div>
			<div class="form-group" style="flex:1">
				<label class="label">Hedef Yol</label>
				<input type="text" bind:value={form.to_path} class="input" placeholder="/yeni-sayfa" />
			</div>
			<div class="form-group" style="width:140px">
				<label class="label">Durum Kodu</label>
				<select bind:value={form.status_code} class="select">
					<option value={301}>301 (Kalıcı)</option>
					<option value={302}>302 (Geçici)</option>
				</select>
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
{:else if redirects.length === 0}
	<p class="text-muted text-center" style="padding:40px">Henüz yönlendirme yok.</p>
{:else}
	<div class="table-wrap">
		<table>
			<thead><tr><th>Kaynak</th><th>Hedef</th><th>Kod</th><th>Tarih</th><th>İşlem</th></tr></thead>
			<tbody>
				{#each redirects as r (r.id)}
					<tr>
						<td>{r.from_path}</td>
						<td>{r.to_path}</td>
						<td><span class="badge">{r.status_code}</span></td>
						<td class="text-muted text-sm">{new Date(r.created_at).toLocaleDateString('tr-TR')}</td>
						<td>
							<button class="btn btn-danger btn-sm" onclick={() => deleteRedirect(r)}>Sil</button>
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
