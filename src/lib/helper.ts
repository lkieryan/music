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
