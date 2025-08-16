import {
	BackgroundRender,
	LyricPlayer,
	type LyricPlayerRef,
} from "@applemusic-like-lyrics/react";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { cn } from "~/lib/helper";
import { AnimatePresence, LayoutGroup } from "framer-motion";
import structuredClone from "@ungap/structured-clone";
import {
	type FC,
	type HTMLProps,
	useEffect,
	useLayoutEffect,
	useMemo,
	useRef,
	useState,
} from "react";

import { toDuration } from "~/lib/helper";
import { AutoLyricLayout } from "~/components/layout/lyric";
import { AudioFFTVisualizer } from "~/components/modules/lyric/audio-fft-visualizer";
import { AudioQualityTag } from "~/components/modules/lyric/audio-quality-tag";
import { BouncingSlider } from "~/components/ui/slider/bouncing-slider";
import { ControlThumb } from "~/components/ui/slider/control-thumb";
import { Cover } from "~/components/ui/cover";
import { MediaButton } from "~/components/ui/button/media-button";
import { MusicInfo } from "~/components/modules/lyric/music-info";
import { PrebuiltToggleIconButton } from "~/components/ui/button/toggleicon-button";
import { PrebuiltToggleIconButtonType } from "~/components/ui/button/toggleicon-button";

import { VolumeControl } from "~/components/ui/slider/volume-control-slider";

import IconForward from "~/assets/icons/icon_forward.svg?react";
import IconPause from "~/assets/icons/icon_pause.svg?react";
import IconPlay from "~/assets/icons/icon_play.svg?react";
import IconRewind from "~/assets/icons/icon_rewind.svg?react";
import RepeatIcon from "~/assets/icons/repeat.svg?react";
import RepeatActiveIcon from "~/assets/icons/repeat-active.svg?react";
import RepeatOneActiveIcon from "~/assets/icons/repeat-one-active.svg?react";
import ShuffleIcon from "~/assets/icons/shuffle.svg?react";
import ShuffleActiveIcon from "~/assets/icons/shuffle-active.svg?react";

import "./icon-animations.css";
import styles from "./index.module.css";
import { useThrottle } from "~/hooks/common/use-throttle";
import React from "react";
import { useLyricsSettingValue } from "~/atoms/settings/lyrics";
import {
	onRequestOpenMenuAtom,
	onRequestPrevSongAtom,
	onRequestNextSongAtom,
	onPlayOrResumeAtom,
	onClickAudioQualityTagAtom,
	onSeekPositionAtom,
	onLyricLineClickAtom,
	onLyricLineContextMenuAtom,
	onChangeVolumeAtom,
} from "~/atoms/player/callbacks";
import {
	showMusicNameAtom,
	showMusicArtistsAtom,
	showMusicAlbumAtom,
	showRemainingTimeAtom,
	showVolumeControlAtom,
	playerControlsTypeAtom,
	PlayerControlsType,
	verticalCoverLayoutAtom,
	lyricBackgroundRendererAtom,
	showBottomControlAtom,
	VerticalCoverLayout,
} from "~/atoms/player/config-atoms";
import {
	musicNameAtom,
	musicArtistsAtom,
	musicAlbumNameAtom,
	musicPlayingAtom,
	musicDurationAtom,
	musicQualityTagAtom,
	musicLyricLinesAtom,
	musicVolumeAtom,
	fftDataAtom,
	hideLyricViewAtom,
	musicCoverAtom,
	musicCoverIsVideoAtom,
	lowFreqVolumeAtom,
	isLyricPageOpenedAtom,
} from "~/atoms/player/data-atoms";

import {
	isShuffleActiveAtom,
	repeatModeAtom,
	RepeatMode,
	toggleShuffleActionAtom,
	cycleRepeatModeActionAtom,
	correctedMusicPlayingPositionAtom,
} from "~/atoms/player/controls-atoms";

const PrebuiltMusicInfo: FC<{
	className?: string;
	style?: React.CSSProperties;
}> = ({ className, style }) => {
	const musicName = useAtomValue(musicNameAtom);
	const musicArtists = useAtomValue(musicArtistsAtom);
	const musicAlbum = useAtomValue(musicAlbumNameAtom);
	const onMenuClicked = useAtomValue(onRequestOpenMenuAtom).onEmit;
	const showMusicName = useAtomValue(showMusicNameAtom);
	const showMusicArtists = useAtomValue(showMusicArtistsAtom);
	const showMusicAlbum = useAtomValue(showMusicAlbumAtom);
	const lyrics = useLyricsSettingValue();
	const fontFamily = lyrics.fontFamily;
	const fontWeight = lyrics.fontWeight;
	const letterSpacing = lyrics.letterSpacing;
	const combinedStyle = useMemo(
		() => ({
			...style,
			fontFamily: fontFamily || undefined,
			fontWeight: fontWeight || undefined,
			letterSpacing: letterSpacing || undefined,
		}),
		[style, fontFamily, fontWeight, letterSpacing],
	);

	return (
		<MusicInfo
			className={className}
			style={combinedStyle}
			name={showMusicName ? musicName : undefined}
			artists={showMusicArtists ? musicArtists.map((v) => v.name) : undefined}
			album={showMusicAlbum ? musicAlbum : undefined}
			onMenuButtonClicked={onMenuClicked}
		/>
	);
};

const PrebuiltMediaButtons: FC<{
	showOtherButtons?: boolean;
}> = ({ showOtherButtons }) => {
	const musicIsPlaying = useAtomValue(musicPlayingAtom);
	const onRequestPrevSong = useAtomValue(onRequestPrevSongAtom).onEmit;
	const onRequestNextSong = useAtomValue(onRequestNextSongAtom).onEmit;
	const onPlayOrResume = useAtomValue(onPlayOrResumeAtom).onEmit;

	const isShuffleOn = useAtomValue(isShuffleActiveAtom);
	const currentRepeatMode = useAtomValue(repeatModeAtom);

	const toggleShuffle = useSetAtom(toggleShuffleActionAtom);
	const cycleRepeat = useSetAtom(cycleRepeatModeActionAtom);

	const iconStyle = {
		width: "1.3em",
		height: "1.3em",
	};

	const renderRepeatIcon = () => {
		switch (currentRepeatMode) {
			case RepeatMode.One:
				return <RepeatOneActiveIcon color="#ffffffff" style={iconStyle} />;
			case RepeatMode.All:
				return <RepeatActiveIcon color="#ffffffff" style={iconStyle} />;
			case RepeatMode.Off:
			default:
				return <RepeatIcon color="#ffffffff" style={iconStyle} />;
		}
	};

	return (
		<>
			{showOtherButtons && (
				<MediaButton className={styles.songMediaButton} onClick={toggleShuffle}>
					{isShuffleOn ? (
						<ShuffleActiveIcon color="#ffffffff" style={iconStyle} />
					) : (
						<ShuffleIcon color="#ffffffff" style={iconStyle} />
					)}
				</MediaButton>
			)}
			<MediaButton
				className={styles.songMediaButton}
				onClick={onRequestPrevSong}
			>
				<IconRewind color="#FFFFFF" />
			</MediaButton>
			<MediaButton
				className={styles.songMediaPlayButton}
				onClick={onPlayOrResume}
			>
				{musicIsPlaying ? (
					<IconPause color="#FFFFFF" />
				) : (
					<IconPlay color="#FFFFFF" />
				)}
			</MediaButton>
			<MediaButton
				className={styles.songMediaButton}
				onClick={onRequestNextSong}
			>
				<IconForward color="#FFFFFF" />
			</MediaButton>

			{showOtherButtons && (
				<MediaButton className={styles.songMediaButton} onClick={cycleRepeat}>
					{renderRepeatIcon()}
				</MediaButton>
			)}
		</>
	);
};

const PrebuiltProgressBar: FC<{ disabled?: boolean }> = React.memo(
	({ disabled }) => {
		const musicDuration = useAtomValue(musicDurationAtom);
		const musicPosition = useAtomValue(correctedMusicPlayingPositionAtom);
		const musicQualityTag = useAtomValue(musicQualityTagAtom);
		const onClickAudioQualityTag = useAtomValue(
			onClickAudioQualityTagAtom,
		).onEmit;
		const onSeekPosition = useAtomValue(onSeekPositionAtom).onEmit;

		const [showRemaining, setShowRemaining] = useAtom(showRemainingTimeAtom);

		const lyrics = useLyricsSettingValue();
		const fontFamily = lyrics.fontFamily as any;
		const fontWeight = lyrics.fontWeight as any;
		const letterSpacing = lyrics.letterSpacing as any;

		const fontStyle = useMemo(
			() => ({
				fontFamily: fontFamily || undefined,
				fontWeight: fontWeight || undefined,
				letterSpacing: letterSpacing || undefined,
			}),
			[fontFamily, fontWeight, letterSpacing],
		);

		const throttledSeek = useThrottle((position: number) => {
			onSeekPosition?.(position);
		}, 100);

		const TimeLabel: FC<{ isRemaining?: boolean }> = ({ isRemaining }) => {
			const currentPosition = useAtomValue(correctedMusicPlayingPositionAtom);
			const duration = useAtomValue(musicDurationAtom);
			const time = isRemaining
				? (currentPosition - duration) / 1000
				: currentPosition / 1000;
			return <>{toDuration(time)}</>;
		};

		const TotalDurationLabel: FC = () => {
			const duration = useAtomValue(musicDurationAtom);
			return <>{toDuration(duration / 1000)}</>;
		};

		return (
			<div>
				<BouncingSlider
					min={0}
					max={musicDuration}
					value={musicPosition}
					onChange={throttledSeek}
					disabled={disabled}
				/>
				<div className={styles.progressBarLabels}>
					<div style={fontStyle}>
						<TimeLabel />
					</div>
					<div>
						<AnimatePresence mode="popLayout">
							{musicQualityTag && (
								<AudioQualityTag
									className={styles.qualityTag}
									isDolbyAtmos={musicQualityTag.isDolbyAtmos}
									tagText={musicQualityTag.tagText}
									tagIcon={musicQualityTag.tagIcon}
									onClick={onClickAudioQualityTag}
								/>
							)}
						</AnimatePresence>
					</div>
					<div
						style={{ ...fontStyle, cursor: "pointer", userSelect: "none" }}
						onClick={() => setShowRemaining(!showRemaining)}
					>
						{showRemaining ? <TimeLabel isRemaining /> : <TotalDurationLabel />}
					</div>
				</div>
			</div>
		);
	},
);

const PrebuiltCoreLyricPlayer: FC<{
	alignPosition: number;
	alignAnchor: "center" | "bottom" | "top";
}> = ({ alignPosition, alignAnchor }) => {
	const amllPlayerRef = useRef<LyricPlayerRef>(null);
	const musicIsPlaying = useAtomValue(musicPlayingAtom);
	const lyricLines = useAtomValue(musicLyricLinesAtom);
	const isLyricPageOpened = useAtomValue(isLyricPageOpenedAtom);
	const musicPlayingPosition = useAtomValue(correctedMusicPlayingPositionAtom);

	const lyrics = useLyricsSettingValue();
	const lyricFontFamily = lyrics.fontFamily;
	const lyricFontWeight = lyrics.fontWeight;
	const lyricLetterSpacing = lyrics.letterSpacing;

	const lyrics2 = useLyricsSettingValue();
	const lyricPlayerImplementation = (lyrics2.playerImplementation) || "dom";

	const enableLyricLineBlurEffect = !!lyrics2.lineBlurEffect;
	const enableLyricLineScaleEffect = !!lyrics2.lineScaleEffect;
	const enableLyricLineSpringAnimation = !!lyrics2.lineSpringAnimation;
	const lyricWordFadeWidth = Number(lyrics2.wordFadeWidth ?? 0.5);
	const enableLyricTranslationLine = !!lyrics2.translationLine;
	const enableLyricRomanLine = !!lyrics2.romanLine;
	const enableLyricSwapTransRomanLine = !!lyrics2.swapTransRomanLine;
	const onLyricLineClick = useAtomValue(onLyricLineClickAtom).onEmit;
	const onLyricLineContextMenu = useAtomValue(
		onLyricLineContextMenuAtom,
	).onEmit;

	const processedLyricLines = useMemo(() => {
		const processed = structuredClone(lyricLines);
		if (!enableLyricTranslationLine) {
			for (const line of processed) {
				line.translatedLyric = "";
			}
		}
		if (!enableLyricRomanLine) {
			for (const line of processed) {
				line.romanLyric = "";
			}
		}
		if (enableLyricSwapTransRomanLine) {
			for (const line of processed) {
				[line.translatedLyric, line.romanLyric] = [
					line.romanLyric,
					line.translatedLyric,
				];
			}
		}
		return processed.map((line: any) => ({
			...line,
			words: Array.isArray(line.words)
				? line.words.map((word: any) => ({
						...word,
						obscene: typeof word.obscene === "boolean" ? word.obscene : false,
					}))
				: [],
		}));
	}, [
		lyricLines,
		enableLyricTranslationLine,
		enableLyricRomanLine,
		enableLyricSwapTransRomanLine,
	]);

	return (
		<LyricPlayer
			style={{
				width: "100%",
				height: "100%",
				fontFamily: lyricFontFamily || undefined,
				fontWeight: lyricFontWeight || undefined,
				letterSpacing: lyricLetterSpacing || undefined,
			}}
			ref={amllPlayerRef}
			playing={musicIsPlaying}
			disabled={!isLyricPageOpened}
			alignPosition={alignPosition}
			alignAnchor={alignAnchor}
			currentTime={musicPlayingPosition}
			lyricLines={processedLyricLines}
			enableBlur={enableLyricLineBlurEffect}
			enableScale={enableLyricLineScaleEffect}
			enableSpring={enableLyricLineSpringAnimation}
			wordFadeWidth={Math.max(0.01, lyricWordFadeWidth)}
			lyricPlayer={lyricPlayerImplementation}
			onLyricLineClick={(evt) => onLyricLineClick?.(evt, amllPlayerRef.current)}
			onLyricLineContextMenu={(evt) =>
				onLyricLineContextMenu?.(evt, amllPlayerRef.current)
			}
		/>
	);
};

const PrebuiltVolumeControl: FC<{
	style?: React.CSSProperties;
	className?: string;
}> = ({ style, className }) => {
	const musicVolume = useAtomValue(musicVolumeAtom);
	const onChangeVolume = useAtomValue(onChangeVolumeAtom).onEmit;
	const showVolumeControl = useAtomValue(showVolumeControlAtom);

	const throttledOnChangeVolume = useThrottle((volume: number) => {
		onChangeVolume?.(volume);
	}, 100);

	if (showVolumeControl)
		return (
			<VolumeControl
				value={musicVolume}
				min={0}
				max={1}
				style={style}
				className={className}
				onChange={throttledOnChangeVolume}
			/>
		);
	return null;
};

const PrebuiltMusicControls: FC<
	{
		showOtherButtons?: boolean;
	} & HTMLProps<HTMLDivElement>
> = ({ className, showOtherButtons, ...props }) => {
	const playerControlsType = useAtomValue(playerControlsTypeAtom);
	const fftData = useAtomValue(fftDataAtom);
	return (
		<div className={cn(styles.controls, className)} {...props}>
			{playerControlsType === PlayerControlsType.Controls && (
				<PrebuiltMediaButtons showOtherButtons={showOtherButtons} />
			)}
			{playerControlsType === PlayerControlsType.FFT && (
				<AudioFFTVisualizer
					style={{
						width: "100%",
						height: "8vh",
					}}
					fftData={fftData}
				/>
			)}
		</div>
	);
};

/**
 * 已经部署好所有组件的歌词播放器组件，在正确设置所有的 Jotai 状态后可以开箱即用
 */
export const PrebuiltLyricPlayer: FC<HTMLProps<HTMLDivElement>> = ({
	className,
	...rest
}) => {
	const [hideLyricView, setHideLyricView] = useAtom(hideLyricViewAtom);
	const musicCover = useAtomValue(musicCoverAtom);
	const musicCoverIsVideo = useAtomValue(musicCoverIsVideoAtom);
	const musicIsPlaying = useAtomValue(musicPlayingAtom);
	const lowFreqVolume = useAtomValue(lowFreqVolumeAtom);
	const isLyricPageOpened = useAtomValue(isLyricPageOpenedAtom);
	const setLyricPageOpened = useSetAtom(isLyricPageOpenedAtom)
	useEffect(() => {
		console.log('isLyricPageOpened changed:', isLyricPageOpened)
	  }, [isLyricPageOpened])
	const lyricsBg1 = useLyricsSettingValue();
	const lyricBackgroundFPS = lyricsBg1.backgroundFps ?? 60;
	const verticalCoverLayout = useAtomValue(verticalCoverLayoutAtom);
	const lyricsBg2 = useLyricsSettingValue();
	const lyricBackgroundStaticMode = !!lyricsBg2.backgroundStaticMode;
	const lyricBackgroundRenderScale = lyricsBg2.backgroundRenderScale ?? 1;
	const [isVertical, setIsVertical] = useState(false);
	const [alignPosition, setAlignPosition] = useState(0.25);
	const [alignAnchor, setAlignAnchor] = useState<"center" | "bottom" | "top">(
		"top",
	);
	const coverElRef = useRef<HTMLDivElement>(null);
	const layoutRef = useRef<HTMLDivElement>(null);
	const lyricsBg3 = useLyricsSettingValue();
	const cssBg = lyricsBg3.backgroundRenderer === "css-bg";
	const cssBgValue = lyricsBg3.cssBackgroundProperty || "#111111";
	const legacyBackgroundRenderer = useAtomValue(lyricBackgroundRendererAtom);
	const backgroundRenderer = cssBg
		? { renderer: cssBgValue }
		: legacyBackgroundRenderer;
	const showBottomControl = useAtomValue(showBottomControlAtom);

	const [isHoveringTitlebar, setIsHoveringTitlebar] = useState(false);
	const [isGracePeriodOver, setIsGracePeriodOver] = useState(false);

	useLayoutEffect(() => {
		// 如果是水平布局，则让歌词对齐到封面的中心
		if (!isVertical && coverElRef.current && layoutRef.current) {
			const obz = new ResizeObserver(() => {
				if (!(coverElRef.current && layoutRef.current)) return;
				const coverB = coverElRef.current.getBoundingClientRect();
				const layoutB = layoutRef.current.getBoundingClientRect();
				setAlignPosition(
					(coverB.top + coverB.height / 2 - layoutB.top) / layoutB.height,
				);
			});
			obz.observe(coverElRef.current);
			obz.observe(layoutRef.current);
			setAlignAnchor("center");
			return () => obz.disconnect();
		}
		// 如果是垂直布局，则把歌词对齐到顶部（歌曲信息下方）
		if (isVertical && layoutRef.current) {
			setAlignPosition(0.1);
			setAlignAnchor("top");
		}
	}, [isVertical]);

	useEffect(() => {
		if (isLyricPageOpened) {
			setIsGracePeriodOver(false);
			const timerId = setTimeout(() => {
				setIsGracePeriodOver(true);
			}, 5000);
			return () => clearTimeout(timerId);
		}
	}, [isLyricPageOpened]);

	useEffect(() => {
		const titlebarArea = document.getElementById("system-titlebar");
		if (!titlebarArea) return;

		const handleMouseEnter = () => setIsHoveringTitlebar(true);
		const handleMouseLeave = () => setIsHoveringTitlebar(false);

		if (isLyricPageOpened) {
			titlebarArea.addEventListener("mouseenter", handleMouseEnter);
			titlebarArea.addEventListener("mouseleave", handleMouseLeave);
		} else {
			setIsHoveringTitlebar(false);
		}

		return () => {
			titlebarArea.removeEventListener("mouseenter", handleMouseEnter);
			titlebarArea.removeEventListener("mouseleave", handleMouseLeave);
		};
	}, [isLyricPageOpened]);

	useEffect(() => {
		const titlebarButtons = document.getElementById("system-titlebar-buttons");
		if (!titlebarButtons) return;

		titlebarButtons.style.transition =
			"opacity 0.3s ease-in-out, pointer-events 0.3s";

		const shouldBeVisible =
			!isLyricPageOpened || isHoveringTitlebar || !isGracePeriodOver;

		titlebarButtons.style.opacity = shouldBeVisible ? "1" : "0";
		titlebarButtons.style.pointerEvents = shouldBeVisible ? "auto" : "none";
	}, [isLyricPageOpened, isHoveringTitlebar, isGracePeriodOver]);

	const verticalImmerseCover =
		hideLyricView &&
		(verticalCoverLayout === VerticalCoverLayout.Auto
			? musicCoverIsVideo && isVertical
			: verticalCoverLayout === VerticalCoverLayout.ForceImmersive);

	return (
		<LayoutGroup>
			<AutoLyricLayout
				ref={layoutRef}
				className={cn(styles.autoLyricLayout, className)}
				onLayoutChange={setIsVertical}
				verticalImmerseCover={verticalImmerseCover}
				coverSlot={
					<Cover
						coverUrl={musicCover}
						coverIsVideo={musicCoverIsVideo}
						ref={coverElRef}
						musicPaused={
							!musicIsPlaying && !musicCoverIsVideo && verticalImmerseCover
						}
					/>
				}
				thumbSlot={<ControlThumb onClick={
					() => setLyricPageOpened(false)
				} />}
				smallControlsSlot={
					<PrebuiltMusicInfo
						className={cn(
							styles.smallMusicInfo,
							hideLyricView && styles.hideLyric,
						)}
					/>
				}
				backgroundSlot={
					typeof backgroundRenderer.renderer === "string" ? (
						<div
							style={{
								zIndex: -1,
								width: "100%",
								height: "100%",
								minWidth: "0",
								minHeight: "0",
								overflow: "hidden",
								background: backgroundRenderer.renderer,
							}}
						/>
					) : (
						<BackgroundRender
							album={musicCover}
							albumIsVideo={musicCoverIsVideo}
							lowFreqVolume={lowFreqVolume}
							renderScale={lyricBackgroundRenderScale}
							fps={lyricBackgroundFPS}
							renderer={backgroundRenderer.renderer}
							staticMode={lyricBackgroundStaticMode || !isLyricPageOpened}
							style={{
								zIndex: -1,
							}}
						/>
					)
				}
				bigControlsSlot={
					<>
						<PrebuiltMusicInfo
							className={cn(
								styles.bigMusicInfo,
								hideLyricView && styles.hideLyric,
							)}
						/>
						<PrebuiltProgressBar />
						<PrebuiltMusicControls className={styles.bigControls} />
						{showBottomControl && (
							<div
								style={{
									display: "flex",
									justifyContent: "space-evenly",
								}}
							>
								<PrebuiltToggleIconButton
									type={PrebuiltToggleIconButtonType.Lyrics}
									checked={!hideLyricView}
									onClick={() => setHideLyricView(!hideLyricView)}
								/>
								<PrebuiltToggleIconButton
									type={PrebuiltToggleIconButtonType.AirPlay}
								/>
								<PrebuiltToggleIconButton
									type={PrebuiltToggleIconButtonType.Playlist}
								/>
							</div>
						)}
						<PrebuiltVolumeControl className={styles.bigVolumeControl} />
					</>
				}
				controlsSlot={
					<>
						<PrebuiltMusicInfo className={styles.horizontalControls} />
						<PrebuiltProgressBar />
						<PrebuiltMusicControls
							className={styles.controls}
							showOtherButtons
						/>
						<PrebuiltVolumeControl />
					</>
				}
				horizontalBottomControls={
					showBottomControl && (
						<>
							<PrebuiltToggleIconButton
								type={PrebuiltToggleIconButtonType.Playlist}
							/>
							<PrebuiltToggleIconButton
								type={PrebuiltToggleIconButtonType.Lyrics}
								checked={!hideLyricView}
								onClick={() => setHideLyricView(!hideLyricView)}
							/>
							<div style={{ flex: "1" }} />
							<PrebuiltToggleIconButton
								type={PrebuiltToggleIconButtonType.AirPlay}
							/>
						</>
					)
				}
				lyricSlot={
					<PrebuiltCoreLyricPlayer
						alignPosition={alignPosition}
						alignAnchor={alignAnchor}
					/>
				}
				hideLyric={hideLyricView}
				{...rest}
			/>
		</LayoutGroup>
	);
};
