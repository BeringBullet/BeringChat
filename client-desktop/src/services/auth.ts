import { login } from './api'
import { invoke } from '@tauri-apps/api/tauri'

// In-memory token fallback (non-persistent). Prefer native secure storage via
// `store_token_secure`/`get_token_secure` when running under Tauri.
export async function doLogin(username: string, password: string) {
  const data = await login(username, password)
  const token = data.access_token || data.token || data.id
  if (token) {
    // keep a runtime-only copy to avoid relying on browser-local storage
    if (typeof window !== 'undefined') (window as any).__BERINGSHARE_TOKEN = token

    // Prefer secure native storage; fall back to legacy `store_token` if necessary.
    try {
      await invoke('store_token_secure', { token })
    } catch (_) {
      try { await invoke('store_token', { token }) } catch (_) {}
    }
  }
  return data
}

export async function logout() {
  if (typeof window !== 'undefined') try { delete (window as any).__BERINGSHARE_TOKEN } catch (_) {}
  try { await invoke('clear_token_secure') } catch (_) {}
  try { await invoke('clear_token') } catch (_) {}
}
