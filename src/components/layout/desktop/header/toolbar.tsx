import type { FC, ReactElement } from 'react'
import { useCallback } from 'react'
import { useNavigate } from 'react-router'
import BackIcon from '~/assets/icons/back.svg?react'
import ForwardIcon from '~/assets/icons/forward.svg?react'
import ReloadIcon from '~/assets/icons/reload.svg?react'
import SaveIcon from '~/assets/icons/save.svg?react'
import SettingsIcon from '~/assets/icons/settings.svg?react'
import MenuIcon from '~/assets/icons/menu.svg?react'

import { useSettingModal } from "~/components/modules/settings/modal/use-setting-modal"
import { WindowControls } from './window-controls'

const iconClassName = "[&_svg_.nc-icon-wrapper]:stroke-current [&_svg_.nc-icon-wrapper]:fill-none [&_svg_.nc-icon-wrapper]:stroke-opacity-100 [&_*[stroke='context-fill']]:stroke-current [&_*[fill='context-fill']]:fill-current [&_*[stroke-opacity='context-fill-opacity']]:stroke-opacity-100"

const ToolbarButton: FC<{
  title: string
  ariaLabel: string
  onClick?: () => void
  icon: ReactElement
}> = ({ title, ariaLabel, onClick, icon }) => {
  return (
    <button 
      className="no-drag inline-flex items-center justify-center w-7 h-7 rounded-[6px] border-none bg-transparent text-current cursor-pointer hover:bg-[rgba(0,0,0,0.06)] dark:hover:bg-[rgba(255,255,255,0.08)] active:bg-[rgba(0,0,0,0.12)] dark:active:bg-[rgba(255,255,255,0.12)]" 
      title={title}
      aria-label={ariaLabel}
      onClick={onClick}
    >
      <span className="w-4 h-4" aria-hidden>
        {icon}
      </span>
    </button>
  )
}

export const LeftToolbar: FC = () => {
  const navigate = useNavigate()
  const goBack = useCallback(() => navigate(-1), [navigate])
  const goForward = useCallback(() => navigate(1), [navigate])
  const onReload = useCallback(() => {
    try {
      navigate(0) // react-router: full document reload
    } catch {
      window.location.reload()
    }
  }, [navigate])
  
  return (
    <div className="inline-flex items-center gap-1.5" aria-label="Left toolbar">
      <ToolbarButton
        title="Back"
        ariaLabel="Back"
        onClick={goBack}
        icon={<BackIcon className={iconClassName} width={16} height={16} />}
      />
      <ToolbarButton
        title="Forward"
        ariaLabel="Forward"
        onClick={goForward}
        icon={<ForwardIcon className={iconClassName} width={16} height={16} />}
      />
      <ToolbarButton
        title="Reload"
        ariaLabel="Reload"
        onClick={onReload}
        icon={<ReloadIcon className={iconClassName} width={16} height={16} />}
      />
    </div>
  )
}

export const RightToolbar: FC = () => {  
  const settingModalPresent = useSettingModal()
  return (
    <div className="inline-flex items-center gap-1.5" aria-label="Right toolbar">
      <ToolbarButton
        title="Downloads"
        ariaLabel="Downloads"
        icon={<SaveIcon className={iconClassName} width={16} height={16} />}
      />
      <ToolbarButton
        title="Settings"
        ariaLabel="Settings"
        onClick={() => settingModalPresent("general")}
        icon={<SettingsIcon className={iconClassName} width={16} height={16} />}
      />
      <ToolbarButton
        title="Menu"
        ariaLabel="Menu"
        icon={<MenuIcon className={iconClassName} width={16} height={16} />}
      />
    </div>
  )
}

export const WindowControlsToolbar: FC = () => {
  return <WindowControls />
}

