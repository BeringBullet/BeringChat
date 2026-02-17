import { invoke } from '@tauri-apps/api/tauri'

export async function connectNativeWs() {
  try {
    await invoke('ws_connect_native')
  } catch (e) {
    console.error('ws_connect_native failed', e)
  }
}

export async function sendViaWs(obj: any) {
  try {
    await invoke('send_ws_message', { message: JSON.stringify(obj) })
    return true
  } catch (e) {
    console.warn('send_ws_message failed, not connected', e)
    return false
  }
}
