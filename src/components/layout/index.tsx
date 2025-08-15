import { withResponsiveComponent } from "~/lib/selector"

export const AppLayout = withResponsiveComponent<object>(
  () =>
    import("~/components/layout/desktop").then((m) => ({
      default: m.DesktopLayout,
    })),
  () =>
    import("~/components/layout/mobile").then((m) => ({
      default: m.MobileLayout,
    })),
)

export const LyricLayout = withResponsiveComponent<object>(
  () =>
    import("~/components/layout/lyric/horizontal").then((m) => ({
      default: m.HorizontalLayout,
    })),
  () =>
      import("~/components/layout/lyric/vertical").then((m) => ({
      default: m.VerticalLayout,
    })),
)

export const AutoLyricLayout =  () => 
  import("~/components/layout/lyric/auto").then((m) => m.AutoLyricLayout)

