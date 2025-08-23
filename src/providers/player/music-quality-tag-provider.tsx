import { useAtomValue, useSetAtom } from "jotai";
import { type FC, useLayoutEffect } from "react";
import { useTranslation } from "react-i18next";

import { musicQualityAtom, musicQualityTagAtom, AudioQualityType } from "~/atoms/player/data-atoms";

/**
 * 处理音频质量并生成对应的标签
 */
function processAudioQuality(quality: any): any {
	const definiteQuality = {
		sampleRate: quality?.sampleRate ?? 0,
		bitsPerCodedSample: quality?.bitsPerCodedSample ?? 0,
		bitsPerSample: quality?.bitsPerSample ?? 0,
		channels: quality?.channels ?? 0,
		sampleFormat: quality?.sampleFormat ?? 'unknown',
		codec: quality?.codec ?? 'unknown'
	};

	if (definiteQuality.codec === 'unknown') {
		return {
			...definiteQuality,
			type: AudioQualityType.None
		};
	}

	const isLosslessCodec = ['flac', 'alac', 'ape', 'wav', 'aiff'].includes(definiteQuality.codec.toLowerCase());

	if (isLosslessCodec) {
		const sampleRate = definiteQuality.sampleRate;
		const bitsPerSample = definiteQuality.bitsPerSample;

		if (sampleRate > 44100 || bitsPerSample > 16) {
			return {
				...definiteQuality,
				type: AudioQualityType.HiResLossless
			};
		}
		return {
			...definiteQuality,
			type: AudioQualityType.Lossless
		};
	}

	return {
		...definiteQuality,
		type: AudioQualityType.None
	};
}

/**
 * 音质标签处理提供者
 * 根据音频质量信息生成对应的标签显示
 */
export const MusicQualityTagProvider: FC = () => {
	const { t } = useTranslation();
	const musicQuality = useAtomValue(musicQualityAtom);
	const setMusicQualityTag = useSetAtom(musicQualityTagAtom);

	useLayoutEffect(() => {
		switch (musicQuality.type) {
			case AudioQualityType.None:
				return setMusicQualityTag(null);

			case AudioQualityType.Lossless:
				return setMusicQualityTag({
					tagIcon: true,
					tagText: t("amll.qualityTag.lossless", "无损"),
					isDolbyAtmos: false,
				});

			case AudioQualityType.HiResLossless:
				return setMusicQualityTag({
					tagIcon: true,
					tagText: t("amll.qualityTag.hires", "高解析度无损"),
					isDolbyAtmos: false,
				});

			case AudioQualityType.DolbyAtmos:
				return setMusicQualityTag({
					tagIcon: false,
					tagText: "",
					isDolbyAtmos: true,
				});

			default:
				return setMusicQualityTag(null);
		}
	}, [t, musicQuality, setMusicQualityTag]);

	return null;
};
