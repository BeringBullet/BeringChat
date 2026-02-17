import React, { useEffect, useState } from 'react'
import { fetchChannelMessages, sendChannelMessage } from '../services/api'
import { sendViaWs } from '../services/ws'

export default function ChatView({ channelId, refreshKey }: { channelId: string; refreshKey?: number }) {
  const [messages, setMessages] = useState<Array<any>>([])
  const [body, setBody] = useState('')

  useEffect(() => {
    fetchChannelMessages(channelId).then((d) => setMessages(d || []))
  }, [channelId, refreshKey])

  async function onSend() {
    if (!body) return
    // Prefer WS for low-latency send; fallback to HTTP
    const envelope = { kind: 'send_channel_message', channel_id: channelId, body }
    const ok = await sendViaWs(envelope)
    if (!ok) await sendChannelMessage(channelId, body)
    setBody('')
    // naive refresh
    fetchChannelMessages(channelId).then((d) => setMessages(d || []))
  }

  return (
    <div className="chat">
      <h3>Channel {channelId}</h3>
      <div>
        {messages.map((m:any) => {
          const sender = m.author_display_name || m.author_username || 'unknown'
          const key = m.message_id || m.id
          return <div key={key}><strong>{sender}</strong>: {m.body}</div>
        })}
      </div>
      <div>
        <input value={body} onChange={e => setBody(e.target.value)} />
        <button onClick={onSend}>Send</button>
      </div>
    </div>
  )
}
