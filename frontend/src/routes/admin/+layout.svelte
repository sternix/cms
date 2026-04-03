<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { auth } from '$lib/stores/auth.js';
	import { theme } from '$lib/stores/theme.js';

	let { children } = $props();
	let authState = $state({ user: null, loading: true, authenticated: false });
	let sidebarOpen = $state(false);
	let currentTheme = $state('light');

	auth.subscribe(v => authState = v);
	theme.subscribe(v => currentTheme = v);

	onMount(async () => {
		theme.init();
		await auth.init();
	});

	$effect(() => {
		if (!authState.loading && !authState.authenticated && !$page.url.pathname.includes('/login')) {
			goto('/admin/login');
		}
	});

	function handleLogout() {
		auth.logout();
		goto('/admin/login');
	}

	const navItems = [
		{ href: '/admin/dashboard', label: 'Gösterge Paneli', icon: '▦' },
		{ href: '/admin/pages', label: 'Sayfalar', icon: '☰' },
		{ href: '/admin/sliders', label: 'Slider', icon: '⊞' },
		{ href: '/admin/media', label: 'Medya', icon: '⊡' },
		{ href: '/admin/menus', label: 'Menüler', icon: '≡' },
		{ href: '/admin/analytics', label: 'Analitik', icon: '◔' },
		{ href: '/admin/redirects', label: 'Yönlendirmeler', icon: '↪' },
		{ href: '/admin/settings', label: 'Ayarlar', icon: '⚙' },
	];
</script>

{#if $page.url.pathname.includes('/login')}
	{@render children()}
{:else if authState.loading}
	<div class="loading-page"><div class="spinner"></div></div>
{:else if authState.authenticated}
	<div class="admin-layout">
		<aside class="sidebar" class:open={sidebarOpen}>
			<div class="sidebar-header">
				<a href="/admin/dashboard" class="sidebar-logo">&#x2699; CMS</a>
				<button class="btn-icon sidebar-close" onclick={() => sidebarOpen = false}>✕</button>
			</div>
			<nav class="sidebar-nav">
				{#each navItems as item}
					<a href={item.href} class="sidebar-link"
					   class:active={$page.url.pathname.startsWith(item.href)}
					   onclick={() => sidebarOpen = false}>
						<span class="sidebar-icon">{item.icon}</span>
						{item.label}
					</a>
				{/each}
			</nav>
			<div class="sidebar-footer">
				<a href="/" class="sidebar-link" target="_blank">
					<span class="sidebar-icon">↗</span> Siteyi Görüntüle
				</a>
			</div>
		</aside>

		{#if sidebarOpen}
			<div class="sidebar-overlay" onclick={() => sidebarOpen = false} role="presentation"></div>
		{/if}

		<div class="admin-main">
			<header class="admin-topbar">
				<button class="btn-icon mobile-menu" onclick={() => sidebarOpen = true}>☰</button>
				<div class="topbar-spacer"></div>
				<div class="topbar-actions">
					<button class="btn btn-ghost btn-sm" onclick={() => theme.toggle()}>
						{currentTheme === 'dark' ? '☀' : '☾'}
					</button>
					<span class="topbar-user">{authState.user?.display_name || authState.user?.username}</span>
					<button class="btn btn-ghost btn-sm" onclick={handleLogout}>Çıkış</button>
				</div>
			</header>
			<div class="admin-content">
				{@render children()}
			</div>
		</div>
	</div>
{/if}

<style>
	.admin-layout {
		display: flex;
		min-height: 100vh;
	}
	.sidebar {
		width: 260px;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		position: fixed;
		top: 0;
		left: 0;
		bottom: 0;
		z-index: 200;
		transition: transform var(--transition-base);
	}
	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border);
	}
	.sidebar-logo {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--color-text);
	}
	.sidebar-close { display: none; }
	.sidebar-nav {
		flex: 1;
		padding: 8px;
		overflow-y: auto;
	}
	.sidebar-link {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: var(--radius-md);
		color: var(--color-text-secondary);
		font-size: 0.9375rem;
		font-weight: 500;
		transition: all var(--transition-fast);
		margin-bottom: 2px;
	}
	.sidebar-link:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}
	.sidebar-link.active {
		background: var(--color-primary-light);
		color: var(--color-primary);
	}
	.sidebar-icon {
		font-size: 1.125rem;
		width: 24px;
		text-align: center;
	}
	.sidebar-footer {
		padding: 8px;
		border-top: 1px solid var(--color-border);
	}
	.sidebar-overlay { display: none; }
	.admin-main {
		flex: 1;
		margin-left: 260px;
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}
	.admin-topbar {
		display: flex;
		align-items: center;
		height: var(--header-height);
		padding: 0 24px;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-surface);
		position: sticky;
		top: 0;
		z-index: 100;
	}
	.mobile-menu { display: none; }
	.topbar-spacer { flex: 1; }
	.topbar-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.topbar-user {
		font-size: 0.875rem;
		color: var(--color-text-secondary);
	}
	.admin-content {
		flex: 1;
		padding: 24px;
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
	}
	.btn-icon:hover {
		background: var(--color-surface-hover);
	}

	@media (max-width: 1024px) {
		.sidebar {
			transform: translateX(-100%);
		}
		.sidebar.open {
			transform: translateX(0);
		}
		.sidebar-close { display: flex; }
		.sidebar-overlay {
			display: block;
			position: fixed;
			inset: 0;
			background: var(--color-overlay);
			z-index: 199;
		}
		.admin-main { margin-left: 0; }
		.mobile-menu { display: flex; }
	}
</style>
