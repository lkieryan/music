import React, { useState } from 'react'

interface CustomColorInputProps {
  onAddColor: (color: string, opacity: number) => void
  className?: string
}

export default function CustomColorInput({ onAddColor, className = '' }: CustomColorInputProps) {
  const [colorValue, setColorValue] = useState('#ffffff')
  const [opacityValue, setOpacityValue] = useState(1)

  const handleAddColor = () => {
    if (!colorValue) return
    
    onAddColor(colorValue, opacityValue)
    setColorValue('#ffffff')
    setOpacityValue(1)
  }

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      handleAddColor()
    }
  }

  return (
    <div className={`theme-picker-color ${className}`} style={{ alignItems: 'start', width: '100%' }}>
      <label style={{ fontSize: '12px', marginLeft: 0, fontWeight: 600, marginBottom: '5px' }}>
        Custom Color
      </label>
      <div style={{ width: '100%', position: 'relative', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <input
          type="color"
          value={colorValue}
          onChange={(e) => setColorValue(e.target.value)}
          onKeyPress={handleKeyPress}
          style={{
            padding: 0,
            alignSelf: 'center',
            border: 'none',
            background: 'transparent',
            width: '40px',
            height: '32px',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        />
        <input
          type="number"
          value={opacityValue}
          min="0"
          max="1"
          step="0.01"
          onChange={(e) => setOpacityValue(parseFloat(e.target.value))}
          onKeyPress={handleKeyPress}
          style={{
            maxWidth: '50px',
            padding: '4px 8px',
            border: '1px solid rgba(0, 0, 0, 0.2)',
            borderRadius: '4px',
            background: 'white',
          }}
          placeholder="1.0"
        />
        <button
          onClick={handleAddColor}
          style={{
            cursor: 'pointer',
            padding: '6px 12px',
            border: 'none',
            background: 'rgba(0, 0, 0, 0.1)',
            borderRadius: '4px',
            fontSize: '12px',
            fontWeight: 600,
            transition: 'background 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'rgba(0, 0, 0, 0.2)'
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'rgba(0, 0, 0, 0.1)'
          }}
        >
          Add
        </button>
      </div>
    </div>
  )
}