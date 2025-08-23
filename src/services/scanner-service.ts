import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Song } from '~/types/bindings'

class ScannerService {
  private isInitialized = false
  private eventListeners: Map<string, Function[]> = new Map()

  constructor() {
    this.initializeService()
  }

  private async initializeService() {
    if (this.isInitialized) return

    try {
      // 监听后端扫描事件
      await listen('scan-progress', (event) => {
        this.emitEvent('scan-progress', event.payload)
      })

      await listen('songs-added', (event) => {
        this.emitEvent('songs-added', event.payload)
      })

      // 应用启动时自动初始化扫描器
      await this.startAutoScanner()
      this.isInitialized = true
    } catch (error) {
      console.error('[ScannerService] initializeService error:', error)
    }
  }

  public on(eventType: string, callback: Function): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, [])
    }
    this.eventListeners.get(eventType)!.push(callback)

    return () => {
      const listeners = this.eventListeners.get(eventType)
      if (listeners) {
        const index = listeners.indexOf(callback)
        if (index > -1) {
          listeners.splice(index, 1)
        }
      }
    }
  }

  private emitEvent(eventType: string, data: any) {
    const listeners = this.eventListeners.get(eventType)
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          console.error(`[ScannerService] emitEvent error (${eventType}):`, error)
        }
      })
    }
  }
  async startAutoScanner(): Promise<void> {
    try {
      await invoke('start_auto_scanner')
      this.emitEvent('scanner-started', null)
    } catch (error) {
      console.error('[ScannerService] startAutoScanner error:', error)
      this.emitEvent('scanner-error', error)
      throw error
    }
  }

  async stopAutoScanner(): Promise<void> {
    try {
      await invoke('stop_auto_scanner')
      this.emitEvent('scanner-stopped', null)
    } catch (error) {
      console.error('[ScannerService] stopAutoScanner error:', error)
      this.emitEvent('scanner-error', error)
      throw error
    }
  }

  async triggerManualScan(paths?: string[]): Promise<void> {
    try {
      await invoke('trigger_manual_scan', paths ? { paths } : undefined)
      this.emitEvent('scan-triggered', { paths })
    } catch (error) {
      console.error('[ScannerService] triggerManualScan error:', error)
      this.emitEvent('scanner-error', error)
      throw error
    }
  }

  async getStatus(): Promise<string> {
    try {
      const status = await invoke<string>('get_auto_scanner_status')
      return status
    } catch (error) {
      console.error('[ScannerService] getStatus error:', error)
      return 'Error'
    }
  }

  async getLocalSongs(): Promise<Song[]> {
    try {
      const songs = await invoke<Song[]>('get_local_songs')
      return songs
    } catch (error) {
      console.error('[ScannerService] getLocalSongs error:', error)
      return [] as Song[]
    }
  }

  async cleanup(): Promise<void> {
    try {
      this.eventListeners.clear()
    } catch (error) {
      console.error('[ScannerService] cleanup error:', error)
    }
  }
}

export const scannerService = new ScannerService()
export default scannerService