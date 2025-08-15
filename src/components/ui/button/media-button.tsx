import { memo, type FC, type HTMLProps, type PropsWithChildren } from "react";
import { cn } from "~/lib/helper";
import styles from "./media-button.module.css";

export const MediaButton: FC<PropsWithChildren<HTMLProps<HTMLButtonElement>>> =
  memo(({ className, children, type, ...rest }) => {
    return (
	  <button
		className={cn(styles.mediaButton, className)}
		type="button"
	    {...rest}
	  >
		{children}
	  </button>
	);
});
