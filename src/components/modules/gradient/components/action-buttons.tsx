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
    <div
      className={`gradient-color-actions ${className}`}
      style={{
        display: 'flex',
        position: 'absolute',
        bottom: '12px',
        left: '50%',
        zIndex: 1,
        transform: 'translateX(-50%)',
        gap: '5px',
      }}
    >
      <button
        onClick={onAddDot}
        disabled={!canAddDot}
        style={{
          border: 'none',
          padding: '0',
          minWidth: 'fit-content',
          transition: 'background 0.2s',
          appearance: 'none',
          maxHeight: '26px',
          maxWidth: '26px',
          minHeight: '26px',
          color: 'rgba(0, 0, 0, 0.7)',
          background: canAddDot ? 'transparent' : 'rgba(0, 0, 0, 0.1)',
          borderRadius: '4px',
          cursor: canAddDot ? 'pointer' : 'not-allowed',
          opacity: canAddDot ? 1 : 0.5,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '16px',
          fontWeight: 'bold',
        }}
        onMouseEnter={(e) => {
          if (canAddDot) {
            e.currentTarget.style.background = 'rgba(0, 0, 0, 0.1)'
          }
        }}
        onMouseLeave={(e) => {
          if (canAddDot) {
            e.currentTarget.style.background = 'transparent'
          }
        }}
        title="Add color dot"
      >
        <PlusIcon width={16} height={16} />
      </button>
      
      <button
        onClick={onRemoveDot}
        disabled={!canRemoveDot}
        style={{
          border: 'none',
          padding: '0',
          minWidth: 'fit-content',
          transition: 'background 0.2s',
          appearance: 'none',
          maxHeight: '26px',
          maxWidth: '26px',
          minHeight: '26px',
          color: 'rgba(0, 0, 0, 0.7)',
          background: canRemoveDot ? 'transparent' : 'rgba(0, 0, 0, 0.1)',
          borderRadius: '4px',
          cursor: canRemoveDot ? 'pointer' : 'not-allowed',
          opacity: canRemoveDot ? 1 : 0.5,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '16px',
          fontWeight: 'bold',
        }}
        onMouseEnter={(e) => {
          if (canRemoveDot) {
            e.currentTarget.style.background = 'rgba(0, 0, 0, 0.1)'
          }
        }}
        onMouseLeave={(e) => {
          if (canRemoveDot) {
            e.currentTarget.style.background = 'transparent'
          }
        }}
        title="Remove color dot"
      >
        <UnpinIcon width={16} height={16} />
      </button>
    </div>
  )
}