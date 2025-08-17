import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ==================================================================
//                            类型定义
// ==================================================================

export interface Song {
  _id?: string;
  path?: string;
  title?: string;
  artist?: string;
  album?: string;
  duration?: number;
  playback_url?: string;
  type_: 'LOCAL' | 'URL' | 'DASH' | 'HLS';
}

export interface QueueItem {
  id: string;
  song: Song;
  added_at: number;
  played_count: number;
}

export interface PlayerState {
  is_playing: boolean;
  is_paused: boolean;
  current_song: Song | null;
  position: number; // 毫秒
  volume: number;
  play_mode: PlayMode;
  queue_index: number | null;
}

export type PlayMode = 'Sequential' | 'RepeatOne' | 'RepeatAll' | 'Shuffle';

export interface PlayerEvent {
  type: 'SongChanged' | 'PlaybackStateChanged' | 'PositionChanged' | 'VolumeChanged' | 'PlayModeChanged' | 'QueueChanged' | 'Error' | 'BufferProgress';
  data: any;
}

// ==================================================================
//                            音频服务类
// ==================================================================

class AudioService {
  private eventListeners: Map<string, Function[]> = new Map();
  private isInitialized = false;

  constructor() {
    this.initializeEventListeners();
  }

  // ============================================================================
  //                              事件处理
  // ============================================================================

  private async initializeEventListeners() {
    if (this.isInitialized) return;

    try {
      // 监听后端播放器事件
      await listen<PlayerEvent>('player-event', (event) => {
        console.log('[AudioService] 收到播放器事件:', event.payload);
        this.emitEvent(event.payload.type, event.payload.data);
      });

      this.isInitialized = true;
      console.log('[AudioService] 事件监听器初始化完成');
    } catch (error) {
      console.error('[AudioService] 初始化事件监听器失败:', error);
    }
  }

  /**
   * 监听特定事件
   */
  public on(eventType: string, callback: Function): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, []);
    }
    this.eventListeners.get(eventType)!.push(callback);

    // 返回取消监听的函数
    return () => {
      const listeners = this.eventListeners.get(eventType);
      if (listeners) {
        const index = listeners.indexOf(callback);
        if (index > -1) {
          listeners.splice(index, 1);
        }
      }
    };
  }

  /**
   * 发射事件给监听器
   */
  private emitEvent(eventType: string, data: any) {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`[AudioService] 事件回调执行失败 (${eventType}):`, error);
        }
      });
    }
  }

  // ============================================================================
  //                              播放控制
  // ============================================================================

  /**
   * 播放指定歌曲
   */
  async playSong(song: Song): Promise<void> {
    try {
      await invoke('play_song', { 
        request: { song }
      });
    } catch (error) {
      console.error('[AudioService] 播放歌曲失败:', error);
      throw error;
    }
  }

  /**
   * 暂停播放
   */
  async pause(): Promise<void> {
    try {
      await invoke('pause_playback');
    } catch (error) {
      console.error('[AudioService] 暂停播放失败:', error);
      throw error;
    }
  }

  /**
   * 恢复播放
   */
  async resume(): Promise<void> {
    try {
      await invoke('resume_playback');
    } catch (error) {
      console.error('[AudioService] 恢复播放失败:', error);
      throw error;
    }
  }

  /**
   * 停止播放
   */
  async stop(): Promise<void> {
    try {
      await invoke('stop_playback');
    } catch (error) {
      console.error('[AudioService] 停止播放失败:', error);
      throw error;
    }
  }

  /**
   * 切换播放/暂停状态
   */
  async togglePlayback(): Promise<boolean> {
    try {
      const isPlaying = await invoke<boolean>('toggle_playback');
      return isPlaying;
    } catch (error) {
      console.error('[AudioService] 切换播放状态失败:', error);
      throw error;
    }
  }

  /**
   * 跳转到指定位置
   */
  async seek(positionSeconds: number): Promise<void> {
    try {
      await invoke('seek_to_position', {
        request: { position_seconds: positionSeconds }
      });
    } catch (error) {
      console.error('[AudioService] 跳转失败:', error);
      throw error;
    }
  }

  /**
   * 设置音量 (0.0 - 1.0)
   */
  async setVolume(volume: number): Promise<void> {
    try {
      await invoke('set_volume', {
        request: { volume }
      });
    } catch (error) {
      console.error('[AudioService] 设置音量失败:', error);
      throw error;
    }
  }

  /**
   * 下一首
   */
  async nextTrack(): Promise<void> {
    try {
      await invoke('next_track');
    } catch (error) {
      console.error('[AudioService] 播放下一首失败:', error);
      throw error;
    }
  }

  /**
   * 上一首
   */
  async previousTrack(): Promise<void> {
    try {
      await invoke('previous_track');
    } catch (error) {
      console.error('[AudioService] 播放上一首失败:', error);
      throw error;
    }
  }

  // ============================================================================
  //                              播放模式和队列
  // ============================================================================

  /**
   * 设置播放模式
   */
  async setPlayMode(mode: PlayMode): Promise<void> {
    try {
      await invoke('set_play_mode', {
        request: { mode }
      });
    } catch (error) {
      console.error('[AudioService] 设置播放模式失败:', error);
      throw error;
    }
  }

  /**
   * 添加歌曲到队列
   */
  async addToQueue(song: Song): Promise<string> {
    try {
      const itemId = await invoke<string>('add_to_queue', {
        request: { song }
      });
      return itemId;
    } catch (error) {
      console.error('[AudioService] 添加到队列失败:', error);
      throw error;
    }
  }

  /**
   * 批量添加歌曲到队列
   */
  async addSongsToQueue(songs: Song[]): Promise<string[]> {
    try {
      const itemIds = await invoke<string[]>('add_songs_to_queue', {
        request: { songs }
      });
      return itemIds;
    } catch (error) {
      console.error('[AudioService] 批量添加到队列失败:', error);
      throw error;
    }
  }

  /**
   * 从队列移除歌曲
   */
  async removeFromQueue(index: number): Promise<QueueItem> {
    try {
      const removedItem = await invoke<QueueItem>('remove_from_queue', {
        request: { index }
      });
      return removedItem;
    } catch (error) {
      console.error('[AudioService] 从队列移除失败:', error);
      throw error;
    }
  }

  /**
   * 获取当前播放队列
   */
  async getQueue(): Promise<QueueItem[]> {
    try {
      const queue = await invoke<QueueItem[]>('get_queue');
      return queue;
    } catch (error) {
      console.error('[AudioService] 获取队列失败:', error);
      throw error;
    }
  }

  /**
   * 清空队列
   */
  async clearQueue(): Promise<void> {
    try {
      await invoke('clear_queue');
    } catch (error) {
      console.error('[AudioService] 清空队列失败:', error);
      throw error;
    }
  }

  /**
   * 播放播放列表
   */
  async playPlaylist(songs: Song[], startIndex?: number): Promise<void> {
    try {
      await invoke('play_playlist', {
        request: { 
          songs, 
          start_index: startIndex 
        }
      });
    } catch (error) {
      console.error('[AudioService] 播放播放列表失败:', error);
      throw error;
    }
  }

  // ============================================================================
  //                              状态查询
  // ============================================================================

  /**
   * 获取播放器状态
   */
  async getPlayerStatus(): Promise<PlayerState> {
    try {
      const status = await invoke<PlayerState>('get_player_status');
      return status;
    } catch (error) {
      console.error('[AudioService] 获取播放器状态失败:', error);
      throw error;
    }
  }

  /**
   * 获取当前播放歌曲
   */
  async getCurrentSong(): Promise<Song | null> {
    try {
      const song = await invoke<Song | null>('get_current_song');
      return song;
    } catch (error) {
      console.error('[AudioService] 获取当前歌曲失败:', error);
      throw error;
    }
  }
}

// ==================================================================
//                            导出单例
// ==================================================================

export const audioService = new AudioService();
export default audioService;