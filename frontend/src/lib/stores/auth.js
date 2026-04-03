import { writable } from 'svelte/store';
import { getAuthToken, setAuthToken, clearAuth, apiGet } from '$lib/utils/api.js';

function createAuthStore() {
	const { subscribe, set } = writable({
		user: null,
		loading: true,
		authenticated: false
	});

	return {
		subscribe,
		async init() {
			const token = getAuthToken();
			if (!token) {
				set({ user: null, loading: false, authenticated: false });
				return;
			}
			try {
				const user = await apiGet('/admin/auth/me');
				set({ user, loading: false, authenticated: true });
			} catch {
				clearAuth();
				set({ user: null, loading: false, authenticated: false });
			}
		},
		login(token, user) {
			setAuthToken(token);
			set({ user, loading: false, authenticated: true });
		},
		logout() {
			clearAuth();
			set({ user: null, loading: false, authenticated: false });
		}
	};
}

export const auth = createAuthStore();
