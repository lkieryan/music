import { cn } from "~/lib/helper";
import { type FC, type HTMLProps, type ReactNode, memo } from "react";
import airplayIcon from "~/assets/icons/airplay.svg?react";
import styles from "./toggleicon-button.module.css";
import lyricsOffIcon from "~/assets/icons/lyrics_off.svg?react";
import lyricsOnIcon from "~/assets/icons/lyrics_on.svg?react";
import playlistOffIcon from "~/assets/icons/playlist_off.svg?react";
import playlistOnIcon from "~/assets/icons/playlist_on.svg?react";
import repeatOffIcon from "~/assets/icons/repeat_off.svg?react";
import repeatOnNormalIcon from "~/assets/icons/repeat_on_normal.svg?react";
import shuffleOffIcon from "~/assets/icons/shuffle_off.svg?react";
import shuffleOnIcon from "~/assets/icons/shuffle_on.svg?react";
import starIcon from "~/assets/icons/star.svg?react";
import starFilledIcon from "~/assets/icons/star_filled.svg?react";

export enum PrebuiltToggleIconButtonType {
	Lyrics = "lyrics",
	Playlist = "playlist",
	Repeat = "repeat",
	Shuffle = "shuffle",
	Star = "star",
	AirPlay = "airplay",
}

export const ToggleIconButton: FC<
	{
		uncheckedIcon: ReactNode;
		checkedIcon: ReactNode;
		checked?: boolean;
	} & Omit<HTMLProps<HTMLButtonElement>, "type">
> = memo(({ uncheckedIcon, checkedIcon, checked, className, ...rest }) => {
	return (
		<button
			className={cn(className, styles.toggleIconButton)}
			type="button"
			{...rest}
		>
			{checked ? checkedIcon : uncheckedIcon}
		</button>
	);
});

type IconComponent = typeof lyricsOffIcon;

const PREBUILT_ICONS_MAP: Record<
	PrebuiltToggleIconButtonType,
	[IconComponent, IconComponent]
> = {
	[PrebuiltToggleIconButtonType.Lyrics]: [lyricsOffIcon, lyricsOnIcon],
	[PrebuiltToggleIconButtonType.Playlist]: [playlistOffIcon, playlistOnIcon],
	[PrebuiltToggleIconButtonType.Repeat]: [repeatOffIcon, repeatOnNormalIcon],
	[PrebuiltToggleIconButtonType.Shuffle]: [shuffleOffIcon, shuffleOnIcon],
	[PrebuiltToggleIconButtonType.Star]: [starIcon, starFilledIcon],
	[PrebuiltToggleIconButtonType.AirPlay]: [airplayIcon, airplayIcon],
};

export const PrebuiltToggleIconButton: FC<
	{
		type: PrebuiltToggleIconButtonType;
		checked?: boolean;
	} & Omit<HTMLProps<HTMLButtonElement>, "type">
> = memo(({ type, checked, onClick, ...rest }) => {
	const [UncheckedIcon, CheckedIcon] = PREBUILT_ICONS_MAP[type];
	return (
		<ToggleIconButton
			uncheckedIcon={<UncheckedIcon />}
			checkedIcon={<CheckedIcon />}
			checked={checked ?? false}
			onClick={onClick}
			{...rest}
		/>
	);
});
