/**
 * Sanitize user input to prevent XSS
 * @param {string} str
 * @returns {string}
 */
export function escapeHtml(str) {
	if (!str) return '';
	return str
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#039;');
}

/**
 * Strip HTML tags from a string
 * @param {string} html
 * @returns {string}
 */
export function stripTags(html) {
	if (!html) return '';
	return html.replace(/<[^>]*>/g, '');
}

/**
 * Validate and sanitize a URL
 * @param {string} url
 * @returns {string}
 */
export function sanitizeUrl(url) {
	if (!url) return '';
	try {
		const parsed = new URL(url, window.location.origin);
		if (['http:', 'https:', 'mailto:'].includes(parsed.protocol)) {
			return url;
		}
		return '';
	} catch {
		if (url.startsWith('/')) return url;
		return '';
	}
}
