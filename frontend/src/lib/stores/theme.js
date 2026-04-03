import { writable } from 'svelte/store';

function createThemeStore() {
	const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('cms_theme') : null;
	const prefersDark = typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;
	const initial = stored || (prefersDark ? 'dark' : 'light');

	const { subscribe, set, update } = writable(initial);

	return {
		subscribe,
		toggle() {
			update(current => {
				const next = current === 'dark' ? 'light' : 'dark';
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem('cms_theme', next);
				}
				if (typeof document !== 'undefined') {
					document.documentElement.setAttribute('data-theme', next);
				}
				return next;
			});
		},
		init() {
			if (typeof document !== 'undefined') {
				const stored = localStorage.getItem('cms_theme');
				const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
				const theme = stored || (prefersDark ? 'dark' : 'light');
				document.documentElement.setAttribute('data-theme', theme);
				set(theme);
			}
		}
	};
}

export const theme = createThemeStore();
