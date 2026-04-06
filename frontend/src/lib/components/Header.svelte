<script>
	import { theme } from "$lib/stores/theme.svelte.js";
	import { onMount } from "svelte";

	let menus = $state([]);
	let settings = $state({ site_name: "CMS" });
	let searchOpen = $state(false);
	let searchQuery = $state("");
	let mobileMenuOpen = $state(false);

	onMount(async () => {
		try {
			const [menuRes, settingsRes] = await Promise.all([
				fetch("/api/admin/menus").catch(() => null),
				fetch("/api/site-settings"),
			]);
			if (settingsRes?.ok) settings = await settingsRes.json();
			// Menus might need auth, fallback to empty
			if (menuRes?.ok) {
				const data = await menuRes.json();
				menus = data.filter((m) => m.is_visible);
			}
		} catch {
			/* fallback defaults */
		}
	});

	function handleSearch(e) {
		e.preventDefault();
		if (searchQuery.trim()) {
			window.location.href = `/search?q=${encodeURIComponent(searchQuery.trim())}`;
			searchOpen = false;
		}
	}
</script>

<header class="site-header">
	<div class="container header-inner">
		<a href="/" class="logo">
			{#if settings.logo_url}
				<img
					src={settings.logo_url}
					alt={settings.site_name}
					class="logo-img"
				/>
			{:else}
				<span class="logo-text">{settings.site_name}</span>
			{/if}
		</a>

		<nav class="nav-desktop">
			{#each menus as menu}
				<a
					href={menu.url}
					class="nav-link"
					target={menu.open_in_new_tab ? "_blank" : undefined}
				>
					{menu.label}
				</a>
			{/each}
		</nav>

		<div class="header-actions">
			<button
				class="btn-icon"
				onclick={() => (searchOpen = !searchOpen)}
				aria-label="Ara"
			>
				⌕
			</button>
			<button
				class="btn-icon"
				onclick={() => theme.toggle()}
				aria-label="Tema değiştir"
			>
				{theme.current === "dark" ? "☀" : "☽"}
			</button>
			<button
				class="btn-icon mobile-menu-btn"
				onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
				aria-label="Menü"
			>
				{mobileMenuOpen ? "✕" : "☰"}
			</button>
		</div>
	</div>

	{#if searchOpen}
		<div class="search-bar">
			<div class="container">
				<form onsubmit={handleSearch} class="search-form">
					<input
						type="search"
						bind:value={searchQuery}
						placeholder="Sayfalarda ara..."
						class="input search-input"
					/>
					<button type="submit" class="btn btn-primary">Ara</button>
				</form>
			</div>
		</div>
	{/if}

	{#if mobileMenuOpen}
		<nav class="nav-mobile">
			{#each menus as menu}
				<a
					href={menu.url}
					class="nav-link"
					onclick={() => (mobileMenuOpen = false)}
					target={menu.open_in_new_tab ? "_blank" : undefined}
				>
					{menu.label}
				</a>
			{/each}
		</nav>
	{/if}
</header>

<style>
	.site-header {
		position: sticky;
		top: 0;
		z-index: 100;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border);
		backdrop-filter: blur(12px);
	}
	.header-inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--header-height);
		gap: 16px;
	}
	.logo {
		display: flex;
		align-items: center;
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--color-text);
		flex-shrink: 0;
	}
	.logo:hover {
		color: var(--color-primary);
	}
	.logo-img {
		height: 36px;
	}
	.nav-desktop {
		display: flex;
		gap: 4px;
	}
	.nav-link {
		padding: 6px 14px;
		border-radius: var(--radius-md);
		color: var(--color-text-secondary);
		font-size: 0.9375rem;
		font-weight: 500;
		transition: all var(--transition-fast);
	}
	.nav-link:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}
	.header-actions {
		display: flex;
		gap: 4px;
		align-items: center;
	}
	.btn-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--color-text-secondary);
		cursor: pointer;
		font-size: 1.25rem;
		transition: all var(--transition-fast);
	}
	.btn-icon:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}
	.mobile-menu-btn {
		display: none;
	}
	.search-bar {
		border-top: 1px solid var(--color-border);
		padding: 12px 0;
		background: var(--color-surface);
	}
	.search-form {
		display: flex;
		gap: 8px;
	}
	.search-input {
		flex: 1;
	}
	.nav-mobile {
		display: none;
	}

	@media (max-width: 768px) {
		.nav-desktop {
			display: none;
		}
		.mobile-menu-btn {
			display: flex;
		}
		.nav-mobile {
			display: flex;
			flex-direction: column;
			background: var(--color-surface);
			border-top: 1px solid var(--color-border);
			padding: 8px 20px 16px;
		}
		.nav-mobile .nav-link {
			padding: 10px 0;
			border-bottom: 1px solid var(--color-border-light);
		}
	}
</style>
