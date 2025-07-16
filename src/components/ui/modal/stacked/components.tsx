import { MotionButtonBase } from "~/components/ui/button"
import { useTranslation } from "react-i18next"

import { useCurrentModal } from "./hooks"
import CloseIcon from "~/assets/icons/close.svg?react"

export const ModalClose = () => {
  const { dismiss } = useCurrentModal()
  const { t } = useTranslation("common")

  return (
    <MotionButtonBase
      aria-label={t("words.close")}
      className="hover:bg-material-ultra-thick absolute right-6 top-6 flex size-8 items-center justify-center rounded-md duration-200"
      onClick={dismiss}
    >
      <CloseIcon className="w-5 h-5" />
    </MotionButtonBase>
  )
}
