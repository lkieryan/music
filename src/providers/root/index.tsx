import type { PropsWithChildren } from 'react'
import { LazyMotion, MotionConfig } from 'motion/react'
import { Spring } from '~/constants/spring'
import { jotaiStore } from '~/lib/jotai'
import { Provider } from "jotai"
import { ThemeProvider } from '~/providers/theme'
import { AppThemeProvider } from '~/providers/root/app-theme-provider'
import { BackgroundProvider } from '~/providers/background'
import { ModalStackProvider } from "~/components/ui/modal"
import { GlobalFocusableProvider } from "~/components/common/focusable/global-focusable-provider"
import { I18nProvider } from '~/providers/root/i18n-provider'

const loadFeatures = () =>
    import('./framer-lazy-feature').then((res) => res.default)


export function RootProvider({ children }: PropsWithChildren) {
  return (
    <Provider store={jotaiStore}>
      <LazyMotion features={loadFeatures} strict key="framer">
        <MotionConfig transition={Spring.presets.smooth}>
          <GlobalFocusableProvider>
            <BackgroundProvider>
              <ModalStackProvider>
                <I18nProvider>
                  <AppThemeProvider>
                    <ThemeProvider>
                      {children}
                    </ThemeProvider>
                  </AppThemeProvider>
                </I18nProvider>
              </ModalStackProvider>
            </BackgroundProvider>
          </GlobalFocusableProvider>
        </MotionConfig>
      </LazyMotion>
    </Provider>
  )
}