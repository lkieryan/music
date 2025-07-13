import type { FocusableProps } from "./focusable"
import { Focusable as FocusableComponent } from "./focusable"

import { FloatingLayerScope, HotkeyScope } from "~/constants/hotkeys"

interface BizFocusableProps extends Omit<FocusableProps, "scope"> {
  scope: HotkeyScope
}
export const Focusable = FocusableComponent as Component<
  Prettify<BizFocusableProps> &
    React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
>

export const FocusablePresets = {
  isNotFloatingLayerScope: (v: Set<string>) => !FloatingLayerScope.some((s) => v.has(s)),
  isSubscriptionList: (scope: Set<string>) => {
    return (
      scope.has(HotkeyScope.SubscriptionList) || (scope.has(HotkeyScope.Home) && scope.size === 1)
    )
  },

  isSubscriptionOrTimeline: (v: Set<string>) => {
    return v.has(HotkeyScope.SubscriptionList) || v.has(HotkeyScope.Timeline) || v.size === 0
  },
  isTimeline: (v) => v.has(HotkeyScope.Timeline) && !v.has(HotkeyScope.EntryRender),
  isEntryRender: (v) => v.has(HotkeyScope.EntryRender),
} satisfies Record<string, (v: Set<string>) => boolean>