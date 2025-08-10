import PlusIcon from '~/assets/icons/plus.svg?react'
import UnpinIcon from '~/assets/icons/unpin.svg?react'

interface ActionButtonsProps {
  canAddDot: boolean
  canRemoveDot: boolean
  onAddDot: () => void
  onRemoveDot: () => void
  className?: string
}

export default function ActionButtons({
  canAddDot,
  canRemoveDot,
  onAddDot,
  onRemoveDot,
  className = ''
}: ActionButtonsProps) {
  return (
    <div className={`flex absolute bottom-3 left-1/2 z-[3] -translate-x-1/2 gap-1.5 ${className}`}>
      <button
        onClick={onAddDot}
        disabled={!canAddDot}
        className={`
          border-none p-0 transition-colors duration-200 appearance-none 
          h-[26px] w-[26px] rounded cursor-pointer 
          flex items-center justify-center text-sm font-bold
          ${canAddDot 
            ? 'text-gray-700 dark:text-gray-300 bg-transparent hover:bg-black/10 dark:hover:bg-white/10 opacity-100' 
            : 'text-gray-500 dark:text-gray-500 bg-black/10 dark:bg-white/10 cursor-not-allowed opacity-50'
          }
        `}
        title="Add color dot"
      >
        <PlusIcon width={16} height={16} />
      </button>
      
      <button
        onClick={onRemoveDot}
        disabled={!canRemoveDot}
        className={`
          border-none p-0 transition-colors duration-200 appearance-none 
          h-[26px] w-[26px] rounded cursor-pointer 
          flex items-center justify-center text-sm font-bold
          ${canRemoveDot 
            ? 'text-gray-700 dark:text-gray-300 bg-transparent hover:bg-black/10 dark:hover:bg-white/10 opacity-100' 
            : 'text-gray-500 dark:text-gray-500 bg-black/10 dark:bg-white/10 cursor-not-allowed opacity-50'
          }
        `}
        title="Remove color dot"
      >
        <UnpinIcon width={16} height={16} />
      </button>
    </div>
  )
}