import React, { useEffect, useState } from 'react'
import { listChannels } from '../services/api'

export default function ChannelList({ onSelect }: { onSelect: (id: string) => void }) {
  const [channels, setChannels] = useState<Array<any>>([])

  useEffect(() => {
    listChannels().then((data) => setChannels(data || []))
  }, [])

  useEffect(() => {
    // listen for presence or channel updates
    // frontend will receive events via Tauri `ws:event`
  }, [])

  return (
    <div className="sidebar">
      <h3>Channels</h3>
      <ul>
        {channels.map((c:any) => (
          <li key={c.id}><button onClick={() => onSelect(c.id)}>{c.name}</button></li>
        ))}
      </ul>
    </div>
  )
}
