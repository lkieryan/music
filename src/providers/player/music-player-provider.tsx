import { useStore } from "jotai";
import { type FC, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import {
    musicAlbumNameAtom,
    musicArtistsAtom,
    musicCoverAtom,
    musicCoverIsVideoAtom,
    musicDurationAtom,
    musicIdAtom,
    musicNameAtom,
    musicPlayingAtom,
    musicPlayingPositionAtom,
    musicVolumeAtom,
    playerModeAtom,
    currentPlaylistAtom,
    currentPlaylistMusicIndexAtom,
    onRequestNextSongAtom,
    onRequestPrevSongAtom,
    onPlayOrResumeAtom,
    isLyricPageOpenedAtom,
    onSeekPositionAtom,
    onLyricLineClickAtom,
    onChangeVolumeAtom,
    onRequestOpenMenuAtom,
} from "~/atoms/player/index";

import { audioService } from "~/services/audio-service";
import type { QueueItem } from "~/services/audio-service";
import type { Song } from "~/types/bindings";
import { resolveSongCoverUrl } from "~/lib/image";

/**
 * music player core provider
 * responsible for player state management, backend event listening and callback injection
 */
export const MusicPlayerProvider: FC = () => {
    const store = useStore();
    const { t } = useTranslation();

    // Track switching flag and timeout to avoid UI flicker when changing tracks
    // switchingRef: true means we are in a manual/auto track switching phase
    const switchingRef = useRef(false);
    const switchingTimerRef = useRef<number | null>(null);

    // Start switching phase with a timeout fallback
    const startSwitching = (reason: 'manual' | 'auto' = 'manual') => {
        // If already switching, refresh timeout
        switchingRef.current = true;
        if (switchingTimerRef.current !== null) {
            clearTimeout(switchingTimerRef.current);
        }
        const timeoutMs = reason === 'manual' ? 3000 : 3000; // Fallback timeout
        switchingTimerRef.current = window.setTimeout(() => {
            switchingRef.current = false;
            switchingTimerRef.current = null;
            console.warn('[Player] switching timeout elapsed, resetting flag');
        }, timeoutMs);
    };

    // Clear switching phase and its timer
    const clearSwitching = () => {
        if (switchingTimerRef.current !== null) {
            clearTimeout(switchingTimerRef.current);
            switchingTimerRef.current = null;
        }
        switchingRef.current = false;
    };

    /**
     * Sync music info to player state
     */
    const syncMusicInfo = async (song: Song) => {
        if (!song) {
            console.error("[syncMusicInfo] Invalid song data, aborting.");
            return;
        }

        try {
            // Set basic song info
            store.set(musicIdAtom, song._id || "");
            store.set(musicNameAtom, song.title || "未知歌曲");
            // Album in bindings is an object; prefer album_name when present
            store.set(musicAlbumNameAtom, song.album?.album_name || "未知专辑");

            // Map artists array to {id,name}
            if (song.artists && song.artists.length > 0) {
                store.set(
                    musicArtistsAtom,
                    song.artists.map((a) => ({
                        id: a.artist_id ?? a.sanitized_artist_name ?? a.artist_name ?? "unknown",
                        name: a.artist_name ?? a.sanitized_artist_name ?? "未知创作者",
                    }))
                );
            } else {
                store.set(musicArtistsAtom, [{ id: "unknown", name: "未知创作者" }]);
            }

            // Clear old blob URL if any
            const oldUrl = store.get(musicCoverAtom);
            if (oldUrl?.startsWith("blob:")) {
                URL.revokeObjectURL(oldUrl);
            }

            // Resolve cover from song/album fields (supports local path or network url)
            const cover = resolveSongCoverUrl(song);
            store.set(
                musicCoverAtom,
                cover ?? "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
            );
            store.set(musicCoverIsVideoAtom, false);

            // Set duration
            if (song.duration) {
                store.set(musicDurationAtom, (song.duration * 1000) | 0);
            }
        } catch (error) {
            console.error("[syncMusicInfo] An error occurred during state update:", error);
        }
    };

    /**
     * Process and set playlist
     */
    const processAndSetPlaylist = async (queueItems: QueueItem[]) => {
        try {
            // Convert backend QueueItem to frontend format
            const playlist = queueItems.map((item, index) => ({
                type: "custom" as const,
                id: item.song._id || "",
                songJsonData: JSON.stringify(item.song),
                origOrder: index,
            }));

            store.set(currentPlaylistAtom, playlist);
        } catch (error) {
            console.error("[processAndSetPlaylist] 处理播放列表失败:", error);
        }
    };

    useEffect(() => {
        // Setup player callback functions
        const toEmitCommand = (command: () => Promise<void>) => ({
            onEmit: async () => {
                try {
                    await command();
                } catch (error) {
                    toast.error("layer command failed:" + (error as Error).message);
                }
            },
        });

        const toEmit = <T,>(onEmit: T) => ({ onEmit });

        // Bind playback control callbacks
        // Intercept next/prev to start local switching phase and debounce rapid clicks
        store.set(onRequestNextSongAtom, {
            onEmit: async () => {
                if (switchingRef.current) return; // Debounce rapid user clicks
                startSwitching('manual');
                try {
                    await audioService.nextTrack();
                } catch (error) {
                    clearSwitching(); // Clear flag on failure
                    console.error("上一首/下一首操作失败:", error);
                    toast.error("操作失败: " + (error as Error).message);
                }
            },
        });
        store.set(onRequestPrevSongAtom, {
            onEmit: async () => {
                if (switchingRef.current) return;
                startSwitching('manual');
                try {
                    await audioService.previousTrack();
                } catch (error) {
                    clearSwitching();
                    toast.error("prev song failed: " + (error as Error).message);
                }
            },
        });
        store.set(onPlayOrResumeAtom, toEmitCommand(async () => { await audioService.togglePlayback(); }));
        
        store.set(isLyricPageOpenedAtom, false);

        store.set(
            onSeekPositionAtom,
            toEmit(async (time: number) => {
                try {
                    await audioService.seek(time / 1000); // Convert to seconds
                } catch (error) {
                    console.error("seek failed:", error);
                }
            }),
        );

        store.set(
            onLyricLineClickAtom,
            toEmit(async (evt: any) => {
                try {
                    // Get time from lyric line data and seek
                    if (evt.line && evt.line.getLine && evt.line.getLine().startTime) {
                        await audioService.seek(evt.line.getLine().startTime / 1000);
                    }
                } catch (error) {
                    console.error("lyric line click failed:", error);
                }
            }),
        );

        store.set(
            onChangeVolumeAtom,
            toEmit(async (volume: number) => {
                try {
                    await audioService.setVolume(volume);
                } catch (error) {
                    console.error("change volume failed:", error);
                }
            }),
        );

        store.set(
            onRequestOpenMenuAtom,
            toEmit(() => {
                toast.info(
                    t("amll.openMenuViaRightClick", "请右键歌词页任意位置来打开菜单哦！"),
                );
            }),
        );

        // Listen to backend player events
        const unsubscribeEvents: (() => void)[] = [];

        // Song changed event
        unsubscribeEvents.push(
            audioService.on("SongChanged", async (data: { song: Song | null }) => {
                console.log("[PlayerEvent] 歌曲改变:", data.song);
                if (data.song) {
                    await syncMusicInfo(data.song);
                    // Clear switching on song metadata arrival
                    clearSwitching();
                    // Also sync current queue index to keep UI highlighting accurate
                    try {
                        const status = await audioService.getPlayerStatus();
                        if (status.queue_index !== null) {
                            store.set(currentPlaylistMusicIndexAtom, status.queue_index);
                        }
                    } catch (err) {
                        console.warn("[SongChanged] Failed to sync queue index:", err);
                    }
                }
            })
        );

        // Playback state changed event
        unsubscribeEvents.push(
            audioService.on("PlaybackStateChanged", (data: { is_playing: boolean; is_paused: boolean }) => {
                // If switching, only land on definitive states and then clear switching
                if (switchingRef.current) {
                    const definitive = data.is_playing || data.is_paused;
                    if (!definitive) return; // ignore non-definitive updates
                    store.set(musicPlayingAtom, Boolean(data.is_playing && !data.is_paused));
                    clearSwitching();
                    return;
                }
                // Normal path
                store.set(musicPlayingAtom, Boolean(data.is_playing && !data.is_paused));
            })
        );

        // Position update event
        unsubscribeEvents.push(
            audioService.on("PositionChanged", (data: { position: { secs: number; nanos: number } }) => {
                // Convert Rust Duration to milliseconds
                const positionMs = data.position.secs * 1000 + Math.floor(data.position.nanos / 1_000_000);
                store.set(musicPlayingPositionAtom, positionMs);
            })
        );

        // Volume changed event
        unsubscribeEvents.push(
            audioService.on("VolumeChanged", (data: { volume: number }) => {
                store.set(musicVolumeAtom, data.volume);
            })
        );

        // Play mode changed event
        unsubscribeEvents.push(
            audioService.on("PlayerModeChanged", (data: { mode: string }) => {
                store.set(playerModeAtom, data.mode as any);
            })
        );

        // Queue changed event
        unsubscribeEvents.push(
            audioService.on("QueueChanged", async () => {
                try {
                    // Sync ordered playlist for UI
                    const queue = await audioService.getQueue();
                    await processAndSetPlaylist(queue);
                    // Sync current index from raw queue
                    try {
                        const raw = await audioService.getQueueRaw();
                        if (Number.isInteger(raw.current_index)) {
                            store.set(currentPlaylistMusicIndexAtom, raw.current_index);
                        }
                    } catch (e) {
                        console.warn("[QueueChanged] Failed to sync queue index:", e);
                    }
                } catch (error) {
                    console.error("get queue failed:", error);
                }
            })
        );

        // Error event
        unsubscribeEvents.push(
            audioService.on("Error", (data: { message: string }) => {
                toast.error("player error: " + data.message);
                clearSwitching();
            })
        );

        // Buffer progress event
        unsubscribeEvents.push(
            audioService.on("BufferProgress", (data: { progress: number }) => {
                // TODO: Handle buffer progress as needed
            })
        );

        // Optional: backend Buffering event to reflect auto switching (no user click)
        unsubscribeEvents.push(
            audioService.on("Buffering", () => {
                // Start switching if not already; helps for auto next-track transitions
                if (!switchingRef.current) startSwitching('auto');
            })
        );

        // Track finished event
        unsubscribeEvents.push(
            audioService.on("TrackFinished", () => {
                clearSwitching();
            })
        );

        // Initialize state on mount
        const initializeState = async () => {
            try {
                const [status, playerMode] = await Promise.all([
                    audioService.getPlayerStatus(),
                    audioService.getPlayerMode(),
                ]);
                // Sync playback state and volume (AggregatedPlayerStatus has: state, current_song, volume, queue_index)
                store.set(musicPlayingAtom, status.state === 'PLAYING');
                store.set(musicVolumeAtom, status.volume);
                store.set(playerModeAtom, playerMode);

                // Sync current song
                if (status.current_song) {
                    await syncMusicInfo(status.current_song);
                }

                // Sync playback queue
                const queue = await audioService.getQueue();
                await processAndSetPlaylist(queue);

                if (status.queue_index !== null) {
                    store.set(currentPlaylistMusicIndexAtom, status.queue_index);
                }
            } catch (error) {
                console.error("[MusicPlayerProvider] init state failed:", error);
            }
        };

        initializeState();

        return () => {
            // Cancel all event listeners
            unsubscribeEvents.forEach(unsubscribe => unsubscribe());

            // Clear switching timer if any
            if (switchingTimerRef.current !== null) {
                clearTimeout(switchingTimerRef.current);
                switchingTimerRef.current = null;
            }

            // Reset callback functions
            const doNothing = toEmit(() => { });
            store.set(onRequestNextSongAtom, doNothing);
            store.set(onRequestPrevSongAtom, doNothing);
            store.set(onPlayOrResumeAtom, doNothing);
            store.set(onSeekPositionAtom, doNothing);
            store.set(onLyricLineClickAtom, doNothing);
            store.set(onChangeVolumeAtom, doNothing);
            store.set(onRequestOpenMenuAtom, doNothing);
        };
    }, [store, t]);

    return null;
};