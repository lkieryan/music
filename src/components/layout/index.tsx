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

