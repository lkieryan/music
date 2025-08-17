import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { extendTailwindMerge } from "tailwind-merge"

const twMerge = extendTailwindMerge({
  extend: {
    theme: {
      text: [
        "largeTitle",
        "title1",
        "title2",
        "title3",
        "headline",
        "body",
        "callout",
        "subheadline",
        "footnote",
        "caption",
      ],
    },
  },
})
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function toDuration(duration: number) {
	const isRemainTime = duration < 0;

	const d = Math.abs(duration | 0);
	const sec = d % 60;
	const min = Math.floor((d - sec) / 60);
	const secText = "0".repeat(2 - sec.toString().length) + sec;

	return `${isRemainTime ? "-" : ""}${min}:${secText}`;
}

