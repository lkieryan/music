import { LINE_PATH, SINE_PATH } from '../components/gradient/constants'

export interface PathPoint {
  x: number
  y: number
  x1?: number
  y1?: number
  x2?: number
  y2?: number
  type: 'M' | 'C' | 'L'
}

export function parseSinePath(pathStr: string): PathPoint[] {
  const points: PathPoint[] = []
  const commands = pathStr.match(/[MCL]\s*[\d\s.\-,]+/g)
  if (!commands) return points

  commands.forEach((command) => {
    const type = command.charAt(0) as 'M' | 'C' | 'L'
    const coordsStr = command.slice(1).trim()
    const coords = coordsStr.split(/[\s,]+/).map(Number)

    switch (type) {
      case 'M':
        points.push({ x: coords[0], y: coords[1], type: 'M' })
        break
      case 'C':
        if (coords.length >= 6 && coords.length % 6 === 0) {
          for (let i = 0; i < coords.length; i += 6) {
            points.push({
              x1: coords[i],
              y1: coords[i + 1],
              x2: coords[i + 2],
              y2: coords[i + 3],
              x: coords[i + 4],
              y: coords[i + 5],
              type: 'C',
            })
          }
        }
        break
      case 'L':
        points.push({ x: coords[0], y: coords[1], type: 'L' })
        break
    }
  })
  return points
}

export function interpolateWavePath(progress: number): string {
  const linePath = LINE_PATH
  const sinePath = SINE_PATH
  const referenceY = 27.395
  const sinePoints = parseSinePath(sinePath)
  
  if (sinePoints.length === 0) {
    return progress < 0.5 ? linePath : sinePath
  }
  if (progress <= 0.001) return linePath
  if (progress >= 0.999) return sinePath
  
  const t = progress
  let newPathData = ''
  
  sinePoints.forEach((p) => {
    switch (p.type) {
      case 'M': {
        const interpolatedY = referenceY + (p.y - referenceY) * t
        newPathData += `M ${p.x} ${interpolatedY} `
        break
      }
      case 'C': {
        const y1 = referenceY + (p.y1! - referenceY) * t
        const y2 = referenceY + (p.y2! - referenceY) * t
        const y = referenceY + (p.y - referenceY) * t
        newPathData += `C ${p.x1} ${y1} ${p.x2} ${y2} ${p.x} ${y} `
        break
      }
      case 'L':
        newPathData += `L ${p.x} ${p.y} `
        break
    }
  })
  
  return newPathData.trim()
}