import type { LyricLine as CoreLyricLine } from "@applemusic-like-lyrics/core";
import {
	type LyricLine,
	parseEslrc,
	/*parseLrc,
	parseLys,
	parseQrc,
	parseTTML,
	parseYrc, */
} from "@applemusic-like-lyrics/lyric";
import { useAtomValue, useSetAtom } from "jotai";
import { type FC, useEffect } from "react";

import {
	musicIdAtom,
	musicLyricLinesAtom,
	hideLyricViewAtom,
} from "~/atoms/player/data-atoms";
import { advanceLyricDynamicLyricTimeAtom } from "~/atoms/settings/lyrics";

type TransLine = {
	[K in keyof CoreLyricLine]: CoreLyricLine[K] extends string ? K : never;
}[keyof CoreLyricLine];

/**
 * 配对翻译或音译歌词到主歌词行
 */
function pairLyric(line: LyricLine, lines: CoreLyricLine[], key: TransLine) {
	if (
		line.words
			.map((v) => v.word)
			.join("")
			.trim().length === 0
	)
		return;

	interface PairedLine {
		startTime: number;
		lineText: string;
		origIndex: number;
		original: CoreLyricLine;
	}

	const processed: PairedLine[] = lines.map((v, i) => ({
		startTime: Math.min(v.startTime, ...v.words.map((v) => v.startTime)),
		origIndex: i,
		lineText: v.words
			.map((v) => v.word)
			.join("")
			.trim(),
		original: v,
	}));

	let nearestLine: PairedLine | undefined;
	for (const coreLine of processed) {
		if (coreLine.lineText.length > 0) {
			if (coreLine.startTime === line.words[0].startTime) {
				nearestLine = coreLine;
				break;
			}
			if (
				nearestLine &&
				Math.abs(nearestLine.startTime - line.words[0].startTime) >
				Math.abs(coreLine.startTime - line.words[0].startTime)
			) {
				nearestLine = coreLine;
			} else if (nearestLine === undefined) {
				nearestLine = coreLine;
			}
		}
	}

	if (nearestLine) {
		const joined = line.words.map((w) => w.word).join("");
		if (nearestLine.original[key].length > 0)
			nearestLine.original[key] += joined;
		else nearestLine.original[key] = joined;
	}
}

export const LyricProvider: FC = () => {
	const musicId = useAtomValue(musicIdAtom);
	const advanceLyricDynamicLyricTime = useAtomValue(advanceLyricDynamicLyricTimeAtom);
	const setLyricLines = useSetAtom(musicLyricLinesAtom);
	const setHideLyricView = useSetAtom(hideLyricViewAtom);

	// TODO: 从后端获取歌曲和歌词数据
	// const track = useLiveQuery(() => db.tracks.get(musicId), [musicId]);

	useEffect(() => {
		// TODO: 暂时禁用 TTML DB 歌词库同步，后续迁移到后端
		// 这部分功能应该移到后端进行处理
		console.log("[LyricProvider] TTML DB 同步功能暂时禁用，等待后端歌词系统实现");
	}, []);

	useEffect(() => {
		// TODO: 从后端获取歌曲歌词数据
		// 目前暂时清空歌词显示，等待后端歌词 API 实现
		
		if (musicId) {
			console.log("[LyricProvider] 歌曲ID改变:", musicId);
			
			// TODO: 调用后端 API 获取歌词
			// const lyricsData = await audioService.getLyrics(musicId);
			
			// 暂时设置为无歌词状态
			setLyricLines([]);
			setHideLyricView(true);
			
			console.log("[LyricProvider] 歌词功能等待后端实现");
		} else {
			// 没有歌曲时清空歌词
			setLyricLines([]);
			setHideLyricView(true);
		}
		
		// 原来的歌词处理逻辑，等后端实现后恢复
		/*
		if (track) {
			try {
				let parsedLyricLines: LyricLine[] = [];
				switch (track.lyricFormat) {
					case "lrc": {
						parsedLyricLines = parseLrc(track.lyric);
						console.log("解析出 LyRiC 歌词", parsedLyricLines);
						break;
					}
					case "eslrc": {
						parsedLyricLines = parseEslrc(track.lyric);
						console.log("解析出 ESLyRiC 歌词", parsedLyricLines);
						break;
					}
					case "yrc": {
						parsedLyricLines = parseYrc(track.lyric);
						console.log("解析出 YRC 歌词", parsedLyricLines);
						break;
					}
					case "qrc": {
						parsedLyricLines = parseQrc(track.lyric);
						console.log("解析出 QRC 歌词", parsedLyricLines);
						break;
					}
					case "lys": {
						parsedLyricLines = parseLys(track.lyric);
						console.log("解析出 Lyricify Syllable 歌词", parsedLyricLines);
						break;
					}
					case "ttml": {
						parsedLyricLines = parseTTML(track.lyric).lines;
						console.log("解析出 TTML 歌词", parsedLyricLines);
						break;
					}
					default: {
						setLyricLines([]);
						setHideLyricView(true);
						return;
					}
				}

				const compatibleLyricLines: CoreLyricLine[] = parsedLyricLines.map(
					(line) => ({
						...line,
						words: line.words.map((word) => ({
							...word,
							obscene: false,
						})),
					}),
				);

				if (track.translatedLrc) {
					try {
						const translatedLyricLines = parseLrc(track.translatedLrc);
						for (const line of translatedLyricLines) {
							pairLyric(
								{
									...line,
									words: line.words.map((word) => ({
										...word,
										obscene: false,
									})),
								},
								compatibleLyricLines,
								"translatedLyric",
							);
						}
						console.log("已匹配翻译歌词");
					} catch (err) {
						console.warn("解析翻译歌词时出现错误", err);
					}
				}

				if (track.romanLrc) {
					try {
						const romanLyricLines = parseLrc(track.romanLrc);
						for (const line of romanLyricLines) {
							pairLyric(
								{
									...line,
									words: line.words.map((word) => ({
										...word,
										obscene: false,
									})),
								},
								compatibleLyricLines,
								"romanLyric",
							);
						}
						console.log("已匹配音译歌词");
					} catch (err) {
						console.warn("解析音译歌词时出现错误", err);
					}
				}

				const processedLines: CoreLyricLine[] = compatibleLyricLines;
				if (advanceLyricDynamicLyricTime) {
					for (const line of processedLines) {
						line.startTime = Math.max(0, line.startTime - 400);
						line.endTime = Math.max(0, line.endTime - 400);
					}
				}

				setLyricLines(processedLines);
				setHideLyricView(processedLines.length === 0);
			} catch (e) {
				console.warn("解析歌词时出现错误", e);
				setLyricLines([]);
				setHideLyricView(true);
			}
		} else {
			setLyricLines([]);
			setHideLyricView(true);
		}
		*/
	}, [musicId, advanceLyricDynamicLyricTime, setLyricLines, setHideLyricView]);

	return null;
};