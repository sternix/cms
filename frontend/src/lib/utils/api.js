const API_BASE = '/api';

/** @type {string|null} */
let csrfToken = null;

/** @type {string|null} */
let authToken = null;

export function setAuthToken(token) {
	authToken = token;
	if (typeof localStorage !== 'undefined') {
		if (token) {
			localStorage.setItem('cms_token', token);
		} else {
			localStorage.removeItem('cms_token');
		}
	}
}

export function getAuthToken() {
	if (authToken) return authToken;
	if (typeof localStorage !== 'undefined') {
		authToken = localStorage.getItem('cms_token');
	}
	return authToken;
}

export function clearAuth() {
	authToken = null;
	csrfToken = null;
	if (typeof localStorage !== 'undefined') {
		localStorage.removeItem('cms_token');
	}
}

async function fetchCsrfToken() {
	if (csrfToken) return csrfToken;
	try {
		const res = await fetch(`${API_BASE}/csrf-token`);
		const data = await res.json();
		csrfToken = data.csrf_token;
		return csrfToken;
	} catch {
		return '';
	}
}

/**
 * @param {string} path
 * @param {RequestInit} [options]
 */
export async function api(path, options = {}) {
	const url = path.startsWith('http') ? path : `${API_BASE}${path}`;
	const headers = new Headers(options.headers || {});

	const token = getAuthToken();
	if (token) {
		headers.set('Authorization', `Bearer ${token}`);
	}

	const method = (options.method || 'GET').toUpperCase();
	if (['POST', 'PUT', 'DELETE'].includes(method)) {
		const csrf = await fetchCsrfToken();
		if (csrf) headers.set('X-CSRF-Token', csrf);
	}

	if (options.body && typeof options.body === 'string') {
		headers.set('Content-Type', 'application/json');
	}

	const res = await fetch(url, { ...options, headers });

	if (res.status === 401) {
		clearAuth();
		if (typeof window !== 'undefined' && window.location.pathname.startsWith('/admin') && window.location.pathname !== '/admin/login') {
			window.location.href = '/admin/login';
		}
	}

	return res;
}

/**
 * @param {string} path
 */
export async function apiGet(path) {
	const res = await api(path);
	if (!res.ok) throw new Error(`API error: ${res.status}`);
	return res.json();
}

/**
 * @param {string} path
 * @param {any} body
 */
export async function apiPost(path, body) {
	return api(path, {
		method: 'POST',
		body: JSON.stringify(body)
	});
}

/**
 * @param {string} path
 * @param {any} body
 */
export async function apiPut(path, body) {
	return api(path, {
		method: 'PUT',
		body: JSON.stringify(body)
	});
}

/**
 * @param {string} path
 */
export async function apiDelete(path) {
	return api(path, { method: 'DELETE' });
}
