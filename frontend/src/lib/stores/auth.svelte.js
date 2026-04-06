import { getAuthToken, setAuthToken, clearAuth, apiGet } from '$lib/utils/api.js';

class AuthStore {
	user = $state(null);
	loading = $state(true);
	authenticated = $state(false);

	async init() {
		const token = getAuthToken();
		if (!token) {
			this.user = null;
			this.loading = false;
			this.authenticated = false;
			return;
		}
		try {
			const user = await apiGet('/admin/auth/me');
			this.user = user;
			this.loading = false;
			this.authenticated = true;
		} catch {
			clearAuth();
			this.user = null;
			this.loading = false;
			this.authenticated = false;
		}
	}

	login(token, user) {
		setAuthToken(token);
		this.user = user;
		this.loading = false;
		this.authenticated = true;
	}

	logout() {
		clearAuth();
		this.user = null;
		this.loading = false;
		this.authenticated = false;
	}
}

export const auth = new AuthStore();
