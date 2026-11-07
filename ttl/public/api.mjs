import { translate } from './i18n.mjs';

const DEFAULT_API_BASE = 'http://localhost:3000';

function resolveApiBase() {
  const manualBase = new URLSearchParams(window.location.search).get('api_base')
    || window.localStorage.getItem('api_base');

  if (manualBase) {
    return manualBase.replace(/\/+$/, '');
  }

  const hostname = window.location.hostname;
  if (['localhost', '127.0.0.1', '0.0.0.0'].includes(hostname)) {
    return DEFAULT_API_BASE;
  }

  const protocol = window.location.protocol === 'https:' ? 'https' : 'http';
  const apiHostname = hostname === 'ttl.harstadmakers.no'
    ? 'ttl-api.harstadmakers.no'
    : hostname;
  return `${protocol}://${apiHostname}:4433`;
}

export const API_BASE = resolveApiBase();

export async function postJson(path, payload) {
  const response = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  const data = await response.json();

  if (!response.ok) {
    const error = new Error(data.error || translate('error.unknown'));
    error.backendMessage = data.error || '';
    throw error;
  }

  return data;
}
