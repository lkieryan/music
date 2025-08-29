import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { MediaContent, PlayerState, PlayerMode } from '~/types/bindings';



// Backend Queue shape from audio-player store
export interface BackendQueue {
  track_queue: string[];
  current_index: number;
  data: Record<string, MediaContent>;
}

// Frontend-facing structures (may be reworked gradually)
export interface QueueItem {
  id: string;
  track: MediaContent;
}

// Use backend RepeatModes for repeat behavior; shuffle is handled via shuffleQueue().

export interface PlayerEventPayload {
  // NOTE: Pass-through type. Consider narrowing once Rust PlayerEvents is finalized.
  type: string;
  data: any;
}

export interface AggregatedPlayerStatus {
  state: PlayerState;
  current_track: MediaContent | null;
  volume: number;
  queue_index: number | null;
}

class AudioService {
  private eventListeners: Map<string, Function[]> = new Map();
  private isInitialized = false;

  constructor() {
    this.initializeEventListeners();
  }

  // Initialize event listeners for backend -> frontend bridge
  private async initializeEventListeners() {
    if (this.isInitialized) return;

    try {
      // Backend emits "audio_event" with PlayerEvents payload
      await listen<PlayerEventPayload>('audio_event', (event) => {
        console.log('[AudioService] 收到播放器事件:', event.payload);
        this.emitEvent(event.payload.type, event.payload.data);
      });

      this.isInitialized = true;
      console.log('[AudioService] 事件监听器初始化完成');
    } catch (error) {
      console.error('[AudioService] 初始化事件监听器失败:', error);
    }
  }

  // Subscribe to a normalized event name
  public on(eventType: string, callback: Function): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, []);
    }
    this.eventListeners.get(eventType)!.push(callback);

    // Unsubscribe function
    return () => {
      const listeners = this.eventListeners.get(eventType);
      if (listeners) {
        const index = listeners.indexOf(callback);
        if (index > -1) listeners.splice(index, 1);
      }
    };
  }

  // Emit normalized events to listeners
  private emitEvent(eventType: string, data: any) {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(data);
        } catch (error) {
          console.error(`[AudioService] 事件回调执行失败 (${eventType}):`, error);
        }
      });
    }
  }

  // -----------------------------
  // Playback Controls (align to backend)
  // -----------------------------

  // Play specific track via unified 'audio_play' to sync store and actually load+play
  async playTrack(track: MediaContent): Promise<void> {
    try {
      await invoke('audio_play', { track });
    } catch (error) {
      console.error('[AudioService] 播放歌曲失败:', error);
      throw error;
    }
  }

  // Pause playback
  async pause(): Promise<void> {
    try {
      await invoke('audio_pause');
    } catch (error) {
      console.error('[AudioService] 暂停播放失败:', error);
      throw error;
    }
  }

  // Resume playback (no track parameter = play current loaded track)
  async play(): Promise<void> {
    try {
      await invoke('audio_play', {});
    } catch (error) {
      console.error('[AudioService] 恢复播放失败:', error);
      throw error;
    }
  }

  // Stop playback
  async stop(): Promise<void> {
    try {
      await invoke('audio_stop');
    } catch (error) {
      console.error('[AudioService] 停止播放失败:', error);
      throw error;
    }
  }

  // Toggle playback based on backend state
  async togglePlayback(): Promise<boolean> {
    try {
      const state = await invoke<PlayerState>('get_player_state');
      console.log("[AudioService] 当前播放状态: ", state);
      if (state === 'PLAYING') {
        console.log("[AudioService] 暂停播放");
        await invoke('audio_pause');
        return false;
      }
      console.log("[AudioService] 恢复播放");
      await invoke('audio_play');
      return true;
    } catch (error) {
      console.error('[AudioService] 切换播放状态失败:', error);
      throw error;
    }
  }

  // Seek to position (seconds)
  async seek(positionSeconds: number): Promise<void> {
    try {
      await invoke('audio_seek', { pos: positionSeconds });
    } catch (error) {
      console.error('[AudioService] 跳转失败:', error);
      throw error;
    }
  }

  // Set volume (0.0 - 1.0)
  async setVolume(volume: number): Promise<void> {
    try {
      await invoke('audio_set_volume', { volume });
    } catch (error) {
      console.error('[AudioService] 设置音量失败:', error);
      throw error;
    }
  }

  // Get volume from backend
  async getVolume(): Promise<number> {
    try {
      const v = await invoke<number>('audio_get_volume');
      return v;
    } catch (error) {
      console.error('[AudioService] 获取音量失败:', error);
      throw error;
    }
  }

  // -----------------------------
  // Queue and Store interactions
  // -----------------------------

  // Add single track to queue (backend expects Vec<MediaContent>)
  async addToQueue(track: MediaContent): Promise<void> {
    try {
      await invoke('add_to_queue', { tracks: [track] });
    } catch (error) {
      console.error('[AudioService] 添加到队列失败:', error);
      throw error;
    }
  }

  // Add multiple tracks to queue
  async addTracksToQueue(tracks: MediaContent[]): Promise<void> {
    try {
      await invoke('add_to_queue', { tracks });
    } catch (error) {
      console.error('[AudioService] 批量添加到队列失败:', error);
      throw error;
    }
  }

  // Remove by index
  async removeFromQueue(index: number): Promise<void> {
    try {
      await invoke('remove_from_queue', { index });
    } catch (error) {
      console.error('[AudioService] 从队列移除失败:', error);
      throw error;
    }
  }

  // Get backend queue (raw)
  async getQueueRaw(): Promise<BackendQueue> {
    try {
      const queue = await invoke<BackendQueue>('get_queue');
      return queue;
    } catch (error) {
      console.error('[AudioService] 获取队列失败:', error);
      throw error;
    }
  }

  // Get queue as list of QueueItem (ordered by track_queue). This is a convenience adapter.
  async getQueue(): Promise<QueueItem[]> {
    const q = await this.getQueueRaw();
    // Build ordered list strictly from track_queue
    return q.track_queue
      .map((id) => ({ id, track: q.data[id] }))
      .filter((x): x is QueueItem => !!x.track);
  }

  // Clear queue
  async clearQueue(): Promise<void> {
    try {
      await invoke('clear_queue');
    } catch (error) {
      console.error('[AudioService] 清空队列失败:', error);
      throw error;
    }
  }

  // Play a playlist: emulate via clear -> add -> play_now
  async playPlaylist(tracks: MediaContent[], startIndex?: number): Promise<void> {
    try {
      await this.clearQueue();
      await this.addTracksToQueue(tracks);
      const idx = typeof startIndex === 'number' ? startIndex : 0;
      const track = tracks[idx];
      if (track) {
        await invoke('play_now', { track });
      }
    } catch (error) {
      console.error('[AudioService] 播放播放列表失败:', error);
      throw error;
    }
  }

  // Shuffle the current queue
  async shuffleQueue(): Promise<void> {
    try {
      await invoke('shuffle_queue');
    } catch (error) {
      console.error('[AudioService] 随机队列失败:', error);
      throw error;
    }
  }

  // Play a specific track immediately (insert next and advance index)
  async playNow(track: MediaContent): Promise<void> {
    try {
      await invoke('play_now', { track });
    } catch (error) {
      console.error('[AudioService] 立即播放失败:', error);
      throw error;
    }
  }

  // Play now by queue index: resolves the track from current queue
  async playNowByIndex(index: number): Promise<void> {
    try {
      const q = await this.getQueueRaw();
      const id = q.track_queue[index];
      const track = id ? q.data[id] : undefined;
      if (!track) return;
      await invoke('play_now', { track });
    } catch (error) {
      console.error('[AudioService] 立即播放(索引)失败:', error);
      throw error;
    }
  }

  // -----------------------------
  // Status queries
  // -----------------------------

  // Aggregate status from multiple backend calls
  async getPlayerStatus(): Promise<AggregatedPlayerStatus> {
    try {
      const [state, track, volume, queue] = await Promise.all([
        invoke<PlayerState>('get_player_state'),
        invoke<MediaContent | null>('get_current_track'),
        invoke<number>('audio_get_volume'),
        invoke<BackendQueue>('get_queue'),
      ]);

      const queue_index = Number.isInteger(queue?.current_index)
        ? queue.current_index
        : null;

      return { state, current_track: track, volume, queue_index };
    } catch (error) {
      console.error('[AudioService] 获取播放器状态失败:', error);
      throw error;
    }
  }

  // Get current track (pass-through)
  async getCurrentTrack(): Promise<MediaContent | null> {
    try {
      const track = await invoke<MediaContent | null>('get_current_track');
      return track;
    } catch (error) {
      console.error('[AudioService] 获取当前歌曲失败:', error);
      throw error;
    }
  }

  // -----------------------------
  // Unsupported legacy methods (explicit)
  // -----------------------------

  // Next/Previous track via newly exposed backend commands
  async nextTrack(): Promise<void> {
    try {
      await invoke('next_track');
    } catch (error) {
      console.error('[AudioService] 下一首失败:', error);
      throw error;
    }
  }

  async previousTrack(): Promise<void> {
    try {
      await invoke('prev_track');
    } catch (error) {
      console.error('[AudioService] 上一首失败:', error);
      throw error;
    }
  }

  // Get current player mode
  async getPlayerMode(): Promise<PlayerMode> {
    try {
      return await invoke<PlayerMode>('get_player_mode');
    } catch (error) {
      console.error('[AudioService] 获取播放模式失败:', error);
      throw error;
    }
  }

  // Set player mode explicitly
  async setPlayerMode(mode: PlayerMode): Promise<void> {
    try {
      await invoke('set_player_mode', { mode });
    } catch (error) {
      console.error('[AudioService] 设置播放模式失败:', error);
      throw error;
    }
  }

  // Toggle player mode (cycle through Sequential -> Single -> Shuffle -> ListLoop)
  async togglePlayerMode(): Promise<void> {
    try {
      await invoke('toggle_player_mode');
    } catch (error) {
      console.error('[AudioService] 切换播放模式失败:', error);
      throw error;
    }
  }

  // Change current queue index directly (true to force metadata reload)
  async changeIndex(newIndex: number, force = true): Promise<void> {
    try {
      await invoke('change_index', { new_index: newIndex, force });
    } catch (error) {
      console.error('[AudioService] 切换队列索引失败:', error);
      throw error;
    }
  }
}

// ==================================================================
//                            导出单例
// ==================================================================

export const audioService = new AudioService();
export default audioService;