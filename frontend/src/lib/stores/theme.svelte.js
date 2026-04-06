class ThemeStore {
	current = $state('light');

	toggle() {
		const next = this.current === 'dark' ? 'light' : 'dark';
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('cms_theme', next);
		}
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', next);
		}
		this.current = next;
	}

	init() {
		if (typeof document !== 'undefined') {
			const stored = localStorage.getItem('cms_theme');
			const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			const theme = stored || (prefersDark ? 'dark' : 'light');
			document.documentElement.setAttribute('data-theme', theme);
			this.current = theme;
		}
	}
}

export const theme = new ThemeStore();
