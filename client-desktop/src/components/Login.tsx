import React, { useState } from 'react'
import { doLogin } from '../services/auth'
import { connectNativeWs } from '../services/ws'
import { invoke } from '@tauri-apps/api/tauri'

export default function Login({ onLogin }: { onLogin: () => void }) {
  const [user, setUser] = useState('')
  const [pass, setPass] = useState('')
  const [serverUrl, setServerUrl] = useState('http://localhost:8080')
  const [loading, setLoading] = useState(false)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    try {
      // store server URL if running in Tauri (best-effort)
      try { await invoke('store_config', { base_url: serverUrl }) } catch {}
      await doLogin(user, pass)
      await connectNativeWs()
      onLogin()
    } catch (e) {
      alert('Login failed')
    } finally { setLoading(false) }
  }

  return (
    <div className="login">
      <h2>Sign in</h2>
      <form onSubmit={submit}>
        <div>
          <input placeholder="server URL" value={serverUrl} onChange={e => setServerUrl(e.target.value)} />
        </div>
        <div>
          <input placeholder="username" value={user} onChange={e => setUser(e.target.value)} />
        </div>
        <div>
          <input type="password" placeholder="password" value={pass} onChange={e => setPass(e.target.value)} />
        </div>
        <div>
          <button disabled={loading} type="submit">{loading ? 'Signing in...' : 'Sign in'}</button>
        </div>
      </form>
    </div>
  )
}
