import { useAtomValue, useStore } from "jotai";
import { type FC, useEffect } from "react";

import {
	fftDataAtom,
	fftDataRangeAtom,
	lowFreqVolumeAtom,
} from "~/atoms/player/data-atoms";


/**
 * FFT 数据处理和低频体感效果提供者
 * 负责实时处理音频频谱数据，生成平滑的低频音量用于歌词动效
 */
export const FFTToLowPassProvider: FC = () => {
	const store = useStore();
	const fftDataRange = useAtomValue(fftDataRangeAtom);

	useEffect(() => {
		// emitAudioThread("setFFTRange", {
		// 	fromFreq: fftDataRange[0],
		// 	toFreq: fftDataRange[1],
		// });
	}, [fftDataRange]);

	useEffect(() => {
		let rafId: number;
		let curValue = 1;
		let lt = 0;
		const gradient: number[] = [];
		let lastSet = 0; // 节流写入 Jotai 的时间戳
		const minUpdateInterval = 50; // 最多每 50ms 写一次（~20fps）

		function amplitudeToLevel(amplitude: number): number {
			const normalizedAmplitude = amplitude / 255;
			const level = 0.5 * Math.log10(normalizedAmplitude + 1);
			return level;
		}

		function calculateGradient(fftData: number[]): number {
			const window = 10;
			const volume =
				(amplitudeToLevel(fftData[0]) + amplitudeToLevel(fftData[1])) * 0.5;
			
			if (gradient.length < window && !gradient.includes(volume)) {
				gradient.push(volume);
				return 0;
			}
			
			gradient.shift();
			gradient.push(volume);

			const maxInInterval = Math.max(...gradient) ** 2;
			const minInInterval = Math.min(...gradient);
			const difference = maxInInterval - minInInterval;
			
			return difference > 0.35 ? maxInInterval : minInInterval * 0.5 ** 2;
		}

		const onFrame = (dt: number) => {
			const fftData = store.get(fftDataAtom);
			const delta = dt - lt;
			const gradient = calculateGradient(fftData);
			const value = gradient;
			const increasing = curValue < value;

			if (increasing) {
				curValue = Math.min(
					value,
					curValue + (value - curValue) * 0.003 * delta,
				);
			} else {
				curValue = Math.max(
					value,
					curValue + (value - curValue) * 0.003 * delta,
				);
			}

			if (Number.isNaN(curValue)) curValue = 1;

			// 节流写入，降低渲染压力
			if (dt - lastSet >= minUpdateInterval) {
				store.set(lowFreqVolumeAtom, curValue);
				lastSet = dt;
			}
			lt = dt;
			rafId = requestAnimationFrame(onFrame);
		};

		rafId = requestAnimationFrame(onFrame);
		return () => {
			cancelAnimationFrame(rafId);
		};
	}, [store]);

	return null;
};
