import { atom } from "jotai"
import { atomWithStorage } from "jotai/utils"
import type { LyricLine } from "@applemusic-like-lyrics/lyric"
import type { PlayerMode } from "~/types/bindings"

// ==================================================================
//                            类型定义
// ==================================================================

/**
 * 定义了播放列表中歌曲的数据结构。
 * - `local`: 代表本地文件歌曲。
 * - `custom`: 代表通过自定义数据源（如API）获取的歌曲。
 */
export type SongData =
	| { type: "local"; filePath: string; origOrder: number }
	| { type: "custom"; id: string; songJsonData: string; origOrder: number };

/**
 * 定义了艺术家信息的标准结构。
 */
export interface ArtistStateEntry {
	name: string;
	id: string;
}

/**
 * 定义了音频质量的类型枚举。
 */
export enum AudioQualityType {
	None = "none",
	Standard = "standard",
	Lossless = "lossless",
	HiResLossless = "hi-res-lossless",
	DolbyAtmos = "dolby-atmos",
}

/**
 * 定义了描述音频质量完整信息的接口。
 */
export interface MusicQualityState {
	type: AudioQualityType;
	codec: string;
	channels: number;
	sampleRate: number;
	sampleFormat: string;
	bitsPerSample?: number;
	bitsPerCodedSample?: number;
}

// ==================================================================
//                        音乐核心数据原子状态
// ==================================================================

/**
 * 当前播放歌曲的唯一标识符。
 */
export const musicIdAtom = atom<string>("");

/**
 * 当前播放的音乐名称。
 */
export const musicNameAtom = atom<string>("未知歌曲");

/**
 * 当前播放的音乐创作者列表。
 */
export const musicArtistsAtom = atom<ArtistStateEntry[]>([
	{ name: "未知创作者", id: "unknown" },
]);

/**
 * 当前播放的音乐所属专辑名称。
 */
export const musicAlbumNameAtom = atom<string>("未知专辑");

/**
 * 当前播放的音乐专辑封面 URL。
 */
export const musicCoverAtom = atom<string>("");

/**
 * 用于快速比较封面是否变化的哈希值。
 */
export const musicCoverHashAtom = atom<number | null>(null);

/**
 * 当前播放的音乐专辑封面资源是否为视频。
 */
export const musicCoverIsVideoAtom = atom<boolean>(false);

/**
 * 当前音乐的总时长，单位为毫秒。
 */
export const musicDurationAtom = atom<number>(0);

/**
 * 当前音乐是否正在播放。
 */
export const musicPlayingAtom = atom<boolean>(false);

/**
 * 当前音乐的播放进度，单位为毫秒。
 */
export const musicPlayingPositionAtom = atom<number>(0);

/**
 * 当前播放的音乐音量大小，范围在 [0.0-1.0] 之间。
 */
export const musicVolumeAtom = atomWithStorage<number>("player.music-volume", 0.5);

/**
 * 当前播放模式：Sequential/Single/Shuffle/ListLoop
 */
export const playerModeAtom = atom<PlayerMode>("Sequential" as PlayerMode);

/**
 * 当前播放的音乐的歌词行数据。
 */
export const musicLyricLinesAtom = atom<LyricLine[]>([]);

/**
 * 是否隐藏歌词视图。
 */
export const hideLyricViewAtom = atom<boolean>(true);



/**
 * 当前音乐的音质信息对象。
 */
export const musicQualityAtom = atom<MusicQualityState>({
	type: AudioQualityType.None,
	codec: "unknown",
	channels: 2,
	sampleRate: 44100,
	sampleFormat: "s16",
});

/**
 * 根据音质信息生成的、用于UI展示的标签内容。
 */
export const musicQualityTagAtom = atom<{
	tagIcon: boolean;
	tagText: string;
	isDolbyAtmos: boolean;
} | null>(null);

/**
 * 当前的播放列表。
 */
export const currentPlaylistAtom = atom<SongData[]>([]);

/**
 * 当前歌曲在播放列表中的索引。
 */
export const currentPlaylistMusicIndexAtom = atom<number>(0);

// ==================================================================
//                        音频可视化相关原子状态
// ==================================================================

/**
 * 用于音频可视化频谱图的实时频域数据。
 */
export const fftDataAtom = atom<number[]>([]);

/**
 * FFT 数据的频率范围设置。
 */
export const fftDataRangeAtom = atom<[number, number]>([0, 22050]);

/**
 * 代表低频部分的音量大小，用于背景脉动等效果。
 */
export const lowFreqVolumeAtom = atom<number>(1);

// ==================================================================
//                        播放器界面状态
// ==================================================================

/**
 * 歌词页面是否已打开。
 */
export const isLyricPageOpenedAtom = atom<boolean>(false);

/**
 * 播放列表卡片是否打开。
 */
export const playlistCardOpenedAtom = atom<boolean>(false);
