import React, { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import Login from './components/Login'
import ChannelList from './components/ChannelList'
import ChatView from './components/ChatView'
import Settings from './components/Settings'

export default function App() {
  const [loggedIn, setLoggedIn] = useState(false)
  const [selectedChannel, setSelectedChannel] = useState<string | null>(null)
  const [refreshToken, setRefreshToken] = useState(0)

  useEffect(() => {
    const unlisten = listen('ws:event', (e) => {
      // handle incoming events from native WS bridge
      let payload: any = e.payload
      if (typeof payload === 'string') {
        try {
          payload = JSON.parse(payload)
        } catch {
          return
        }
      }
      if (payload?.event === 'new_message') {
        if (payload.channel_id && selectedChannel && payload.channel_id === selectedChannel) {
          setRefreshToken((v) => v + 1)
        }
      }
    })
    return () => { unlisten.then(f => f()) }
  }, [selectedChannel])

  if (!loggedIn) return <Login onLogin={() => setLoggedIn(true)} />

  return (
    <div className="app">
      <div style={{display:'flex', flexDirection:'column'}}>
        <div style={{padding:8, borderBottom:'1px solid #eee'}}>BeringShare Desktop</div>
        <ChannelList onSelect={(id) => setSelectedChannel(id)} />
        <Settings />
      </div>
      {selectedChannel ? <ChatView channelId={selectedChannel} refreshKey={refreshToken} /> : <div className="placeholder">Select a channel</div>}
    </div>
  )
}
