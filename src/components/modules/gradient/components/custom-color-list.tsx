interface CustomColorListProps {
  colors: string[]
  onRemoveColor: (color: string) => void
  className?: string
}

export default function CustomColorList({ colors, onRemoveColor, className = '' }: CustomColorListProps) {
  if (colors.length === 0) {
    return null
  }

  return (
    <div className={`gradient-custom-list ${className}`} style={{ marginTop: '15px' }}>
      {colors.map((color, index) => (
        <div
          key={index}
          className="theme-picker-custom-list-item"
          data-color={color}
          style={{
            display: 'flex',
            padding: '5px',
            position: 'relative',
            alignItems: 'center',
          }}
        >
          <div
            className="theme-picker-dot custom"
            style={{
              background: color,
              border: '1px solid rgba(0, 0, 0, 0.2)',
              borderRadius: '5px',
              width: '20px',
              height: '20px',
              marginRight: '10px',
            }}
          />
          <label
            className="theme-picker-custom-list-item-label"
            style={{
              fontSize: '12px',
              fontWeight: 600,
              margin: 0,
              display: 'flex',
              alignItems: 'center',
              flex: 1,
            }}
          >
            {color}
          </label>
          <button
            className="theme-picker-custom-list-item-remove"
            onClick={() => onRemoveColor(color)}
            style={{
              padding: '2px 4px',
              margin: 0,
              marginLeft: 'auto',
              transition: 'opacity 0.1s',
              opacity: 0,
              border: 'none',
              background: 'rgba(255, 0, 0, 0.1)',
              borderRadius: '3px',
              cursor: 'pointer',
              fontSize: '12px',
              color: 'red',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'rgba(255, 0, 0, 0.2)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(255, 0, 0, 0.1)'
            }}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
            </svg>
          </button>
        </div>
      ))}
      
      <style jsx>{`
        .theme-picker-custom-list-item:hover .theme-picker-custom-list-item-remove {
          opacity: 1;
        }
      `}</style>
    </div>
  )
}