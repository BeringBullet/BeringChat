import axios from 'axios'
import type { AxiosRequestConfig } from 'axios'
import { invoke } from '@tauri-apps/api/tauri'

export const http = axios.create()

http.interceptors.request.use(async (config: AxiosRequestConfig) => {
  try {
    // Prefer a secure native getter if available, fall back to legacy getter
    let token: string | null = null
    try { token = await invoke<string>('get_token_secure') } catch (_) {}
    if (!token) {
      try { token = await invoke<string>('get_token') } catch (_) {}
    }
    // Final fallback: in-memory token set by the auth helper (non-persistent)
    if (!token && typeof window !== 'undefined') token = (window as any).__BERINGSHARE_TOKEN || null
    if (token) {
      config.headers = config.headers || {}
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore allow dynamic header assignment
      config.headers['Authorization'] = `Bearer ${token}`
    }
  } catch (_) {}

  try {
    const base = await invoke<string>('get_config')
    if (base) config.baseURL = base
  } catch (_) {
    if (!config.baseURL) config.baseURL = 'http://localhost:8080'
  }

  return config
})

export async function login(username: string, password: string) {
  const res = await http.post('/api/login', { username, password })
  return res.data
}

export async function listChannels() {
  const res = await http.get('/api/channels')
  return res.data
}

export async function listUsers() {
  const res = await http.get('/api/users')
  return res.data
}

export async function fetchChannelMessages(channelId: string) {
  const res = await http.get(`/api/channels/${channelId}/messages`)
  return res.data
}

export async function sendChannelMessage(channelId: string, body: string) {
  return http.post(`/api/channels/${channelId}/messages`, { body })
}
