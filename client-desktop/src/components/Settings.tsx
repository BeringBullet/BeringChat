import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

export default function Settings() {
  const [base, setBase] = useState('')
  const [status, setStatus] = useState('unknown')

  useEffect(() => {
    invoke<string>('get_config').then(b => setBase(b)).catch(() => {})
    invoke<string>('get_ws_status').then(s => setStatus(s)).catch(() => {})
    // listen for status events
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen('ws:status', (e:any) => setStatus(e.payload as string))
    })
  }, [])

  async function save() {
    try {
      await invoke('store_config', { base_url: base })
      alert('Saved')
    } catch (e) { alert('Failed to save') }
  }

  return (
    <div style={{padding:12}}>
      <h4>Settings</h4>
      <div>
        <label>Server URL</label>
        <input value={base} onChange={e => setBase(e.target.value)} style={{width:'100%'}} />
      </div>
      <div style={{marginTop:8}}>
        <button onClick={save}>Save</button>
      </div>
      <div style={{marginTop:12}}>WS status: <strong>{status}</strong></div>
    </div>
  )
}
