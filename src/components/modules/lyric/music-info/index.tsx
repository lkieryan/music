import { cn } from "~/lib/helper";
import type { HTMLProps } from "react";
import { TextMarquee } from "~/components/ui/text-marquee";
import MenuIcon from '~/assets/icons/menu.svg?react'
import styles from "./index.module.css";

export const MusicInfo: React.FC<{
  name?: string;
  artists?: string[];
  album?: string;
  onArtistClicked?: (artist: string, index: number) => void;
  onAlbumClicked?: () => void;
  onMenuButtonClicked?: () => void;
} & HTMLProps<HTMLDivElement>> = ({
  name,
  artists,
  album, 
  onArtistClicked,
  onAlbumClicked,
  onMenuButtonClicked,
  className,
  ...rest
}) => {
  return (
	<div className={cn(styles.musicInfo, className)} {...rest}>
	  <div className={styles.info}>
		{name !== undefined && (
		  <TextMarquee className={styles.name}>{name}</TextMarquee>
		)}
		{artists !== undefined && (
		  <TextMarquee className={styles.artists}>
			{artists.map((v) => (
			  <a key={`artist-${v}`}>{v}</a>
			))}
		  </TextMarquee>
		)}
		{album !== undefined && (
		  <TextMarquee className={styles.album}>{album}</TextMarquee>
		)}
	  </div>
	  <button
		className={cn(styles.menuButton, className)}
		type="button"
		onClick={onMenuButtonClicked}
	  >
		<MenuIcon />
	  </button>
	</div>
  );
};
