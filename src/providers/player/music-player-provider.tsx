import { useStore } from "jotai";
import { type FC, useEffect } from "react";
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
	musicQualityAtom,
	musicVolumeAtom,
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
import type { Song, PlayerEvent, QueueItem } from "~/services/audio-service";

/**
 * 处理音频质量并生成对应的音质状态
 */
function processAudioQuality(quality: any): any {
	// TODO: 实现音质处理逻辑，根据后端 AudioQuality 结构调整
	return { type: "None" };
}

/**
 * 音乐播放器核心提供者
 * 负责播放器状态管理、后端事件监听和回调注入
 */
export const MusicPlayerProvider: FC = () => {
	const store = useStore();
	const { t } = useTranslation();

	/**
	 * 同步音乐信息到播放器状态
	 */
	const syncMusicInfo = async (song: Song) => {
		if (!song) {
			console.error("[syncMusicInfo] Invalid song data, aborting.");
			return;
		}

		try {
			// 设置歌曲基本信息
			store.set(musicIdAtom, song._id || "");
			store.set(musicNameAtom, song.title || "未知歌曲");
			store.set(musicAlbumNameAtom, song.album || "未知专辑");
			
			// 处理艺术家信息
			if (song.artist) {
				store.set(
					musicArtistsAtom,
					song.artist.split("/").map((v: string) => ({
						id: v.trim(),
						name: v.trim(),
					})),
				);
			} else {
				store.set(musicArtistsAtom, [{ id: "unknown", name: "未知创作者" }]);
			}

			// 清理旧的封面URL
			const oldUrl = store.get(musicCoverAtom);
			if (oldUrl?.startsWith("blob:")) {
				URL.revokeObjectURL(oldUrl);
			}

			// TODO: 从后端获取封面数据
			// 目前先设置默认封面
			store.set(
				musicCoverAtom,
				"data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
			);
			store.set(musicCoverIsVideoAtom, false);

			// 设置时长
			if (song.duration) {
				store.set(musicDurationAtom, (song.duration * 1000) | 0);
			}
		} catch (error) {
			console.error("[syncMusicInfo] An error occurred during state update:", error);
		}
	};

	/**
	 * 处理并设置播放列表
	 */
	const processAndSetPlaylist = async (queueItems: QueueItem[]) => {
		try {
			// 将后端的 QueueItem 转换为前端需要的格式
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
		console.log("[MusicPlayerProvider] 初始化音频服务连接");

		// 设置播放器回调函数
		const toEmitCommand = (command: () => Promise<void>) => ({
			onEmit: async () => {
				try {
					await command();
				} catch (error) {
					console.error("播放器命令执行失败:", error);
					toast.error("操作失败: " + (error as Error).message);
				}
			},
		});

		const toEmit = <T,>(onEmit: T) => ({ onEmit });

		// 绑定播放控制回调
		store.set(onRequestNextSongAtom, toEmitCommand(() => audioService.nextTrack()));
		store.set(onRequestPrevSongAtom, toEmitCommand(() => audioService.previousTrack()));
		store.set(onPlayOrResumeAtom, toEmitCommand(() => audioService.togglePlayback()));
		
		store.set(
			isLyricPageOpenedAtom, false
		);

		store.set(
			onSeekPositionAtom,
			toEmit(async (time: number) => {
				try {
					await audioService.seek(time / 1000); // 转换为秒
				} catch (error) {
					console.error("跳转失败:", error);
				}
			}),
		);

		store.set(
			onLyricLineClickAtom,
			toEmit(async (evt: any) => {
				try {
					// 从歌词行数据中获取时间并跳转
					if (evt.line && evt.line.getLine && evt.line.getLine().startTime) {
						await audioService.seek(evt.line.getLine().startTime / 1000);
					}
				} catch (error) {
					console.error("歌词行跳转失败:", error);
				}
			}),
		);

		store.set(
			onChangeVolumeAtom,
			toEmit(async (volume: number) => {
				try {
					await audioService.setVolume(volume);
				} catch (error) {
					console.error("设置音量失败:", error);
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


		// 监听后端播放器事件
		const unsubscribeEvents: (() => void)[] = [];

		// 歌曲改变事件
		unsubscribeEvents.push(
			audioService.on("SongChanged", async (data: { song: Song | null }) => {
				console.log("[PlayerEvent] 歌曲改变:", data.song);
				if (data.song) {
					await syncMusicInfo(data.song);
				}
			})
		);

		// 播放状态改变事件
		unsubscribeEvents.push(
			audioService.on("PlaybackStateChanged", (data: { is_playing: boolean; is_paused: boolean }) => {
				console.log("[PlayerEvent] 播放状态改变:", data);
				store.set(musicPlayingAtom, data.is_playing);
			})
		);

		// 播放位置更新事件
		unsubscribeEvents.push(
			audioService.on("PositionChanged", (data: { position: { secs: number; nanos: number } }) => {
				// 将 Rust Duration 转换为毫秒
				const positionMs = data.position.secs * 1000 + Math.floor(data.position.nanos / 1_000_000);
				store.set(musicPlayingPositionAtom, positionMs);
			})
		);

		// 音量改变事件
		unsubscribeEvents.push(
			audioService.on("VolumeChanged", (data: { volume: number }) => {
				console.log("[PlayerEvent] 音量改变:", data.volume);
				store.set(musicVolumeAtom, data.volume);
			})
		);

		// 播放模式改变事件
		unsubscribeEvents.push(
			audioService.on("PlayModeChanged", (data: { mode: string }) => {
				console.log("[PlayerEvent] 播放模式改变:", data.mode);
				// TODO: 根据需要更新播放模式状态
			})
		);

		// 队列改变事件
		unsubscribeEvents.push(
			audioService.on("QueueChanged", async () => {
				console.log("[PlayerEvent] 队列改变");
				try {
					const queue = await audioService.getQueue();
					await processAndSetPlaylist(queue);
				} catch (error) {
					console.error("获取队列失败:", error);
				}
			})
		);

		// 错误事件
		unsubscribeEvents.push(
			audioService.on("Error", (data: { message: string }) => {
				console.error("[PlayerEvent] 播放器错误:", data.message);
				toast.error("播放器错误: " + data.message);
			})
		);

		// 缓冲进度事件
		unsubscribeEvents.push(
			audioService.on("BufferProgress", (data: { progress: number }) => {
				console.log("[PlayerEvent] 缓冲进度:", data.progress);
				// TODO: 根据需要处理缓冲进度
			})
		);

		// 初始化时获取当前状态
		const initializeState = async () => {
			try {
				const status = await audioService.getPlayerStatus();
				console.log("[MusicPlayerProvider] 初始状态:", status);

				// 同步播放状态
				store.set(musicPlayingAtom, status.is_playing);
				store.set(musicVolumeAtom, status.volume);
				store.set(musicPlayingPositionAtom, status.position);

				// 同步当前歌曲
				if (status.current_song) {
					await syncMusicInfo(status.current_song);
				}

				// 同步播放队列
				const queue = await audioService.getQueue();
				await processAndSetPlaylist(queue);

				if (status.queue_index !== null) {
					store.set(currentPlaylistMusicIndexAtom, status.queue_index);
				}
			} catch (error) {
				console.error("[MusicPlayerProvider] 初始化状态失败:", error);
			}
		};

		initializeState();

		console.log("[MusicPlayerProvider] 音频服务连接完成");

		return () => {
			console.log("[MusicPlayerProvider] 清理事件监听");
			
			// 取消所有事件监听
			unsubscribeEvents.forEach(unsubscribe => unsubscribe());

			// 重置回调函数
			const doNothing = toEmit(() => { });
			store.set(onRequestNextSongAtom, doNothing);
			store.set(onRequestPrevSongAtom, doNothing);
			store.set(onPlayOrResumeAtom, doNothing);
			store.set(onSeekPositionAtom, doNothing);
			store.set(onLyricLineClickAtom, doNothing);
			store.set(onChangeVolumeAtom, doNothing);
			store.set(onRequestOpenMenuAtom, doNothing);

			console.log("[MusicPlayerProvider] 清理完成");
		};
	}, [store, t]);

	return null;
};