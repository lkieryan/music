export const MAX_OPACITY = 0.9
export const MIN_OPACITY_MACOS = 0.25
export const MIN_OPACITY_OTHER = 0.35
export const MAX_DOTS = 3
export const EXPLICIT_LIGHTNESS_TYPE = 'explicit-lightness'

// Platform detection (using navigator for web)
export const IS_MACOS = typeof navigator !== 'undefined' && /Mac|iPod|iPhone|iPad/.test(navigator.platform)
export const MIN_OPACITY = IS_MACOS ? MIN_OPACITY_MACOS : MIN_OPACITY_OTHER

// Exact SVG paths from original implementation
export const LINE_PATH = 'M 51.373 27.395 L 367.037 27.395'
export const SINE_PATH = 'M 51.373 27.395 C 60.14 -8.503 68.906 -8.503 77.671 27.395 C 86.438 63.293 95.205 63.293 103.971 27.395 C 112.738 -8.503 121.504 -8.503 130.271 27.395 C 139.037 63.293 147.803 63.293 156.57 27.395 C 165.335 -8.503 174.101 -8.503 182.868 27.395 C 191.634 63.293 200.4 63.293 209.167 27.395 C 217.933 -8.503 226.7 -8.503 235.467 27.395 C 244.233 63.293 252.999 63.293 261.765 27.395 C 270.531 -8.503 279.297 -8.503 288.064 27.395 C 296.83 63.293 305.596 63.293 314.363 27.395 C 323.13 -8.503 331.896 -8.503 340.663 27.395 C 349.43 63.293 358.196 63.293 366.963 27.395'

// Color harmonies
export const COLOR_HARMONIES = [
  { type: 'complementary', angles: [180] },
  { type: 'splitComplementary', angles: [150, 210] },
  { type: 'analogous', angles: [50, 310] },
  { type: 'triadic', angles: [120, 240] },
  { type: 'floating', angles: [] },
] as const

// Gradient picker dimensions (338px as per original)
export const PICKER_SIZE = 338
export const PICKER_PADDING = 30
export const DOT_SIZE = 18
export const PRIMARY_DOT_SIZE = 36

// Preset colors data - exact from original XUL
export const PRESET_COLORS = [
  // Page 1 - Light colors, single dots
  [
    { lightness: 90, algo: 'float', numDots: 1, position: '240,240', style: 'background: #f4efdf;' },
    { lightness: 80, algo: 'float', numDots: 1, position: '233,157', style: 'background: #f0b8cd;' },
    { lightness: 80, algo: 'float', numDots: 1, position: '236,111', style: 'background: #e9c3e3;' },
    { lightness: 70, algo: 'float', numDots: 1, position: '234,173', style: 'background: #da7682;' },
    { lightness: 70, algo: 'float', numDots: 1, position: '220,187', style: 'background: #eb8570;' },
    { lightness: 60, algo: 'float', numDots: 1, position: '225,237', style: 'background: #dcce7f;' },
    { lightness: 60, algo: 'float', numDots: 1, position: '147,195', style: 'background: #5becad;' },
    { lightness: 50, algo: 'float', numDots: 1, position: '81,84', style: 'background: #919bb5;' },
  ],
  // Page 2 - Light colors, analogous
  [
    { lightness: 90, algo: 'analogous', numDots: 3, position: '240,240', colors: ['rgb(245, 237, 214)', 'rgb(221, 243, 216)', 'rgb(243, 216, 225)'] },
    { lightness: 85, algo: 'analogous', numDots: 3, position: '233,157', colors: ['rgb(243, 190, 222)', 'rgb(247, 222, 186)', 'rgb(223, 195, 238)'] },
    { lightness: 80, algo: 'analogous', numDots: 3, position: '236,111', colors: ['rgb(229, 179, 228)', 'rgb(236, 172, 178)', 'rgb(197, 185, 223)'] },
    { lightness: 70, algo: 'analogous', numDots: 3, position: '234,173', colors: ['rgb(235, 122, 159)', 'rgb(239, 239, 118)', 'rgb(210, 133, 224)'] },
    { lightness: 70, algo: 'analogous', numDots: 3, position: '220,187', colors: ['rgb(242, 115, 123)', 'rgb(175, 242, 115)', 'rgb(230, 125, 232)'] },
    { lightness: 60, algo: 'analogous', numDots: 3, position: '225,237', colors: ['rgb(221, 205, 85)', 'rgb(97, 212, 94)', 'rgb(215, 91, 124)'] },
    { lightness: 60, algo: 'analogous', numDots: 3, position: '147,195', colors: ['rgb(75, 231, 210)', 'rgb(84, 175, 222)', 'rgb(62, 244, 112)'] },
    { lightness: 55, algo: 'analogous', numDots: 3, position: '81,84', colors: ['rgb(122, 132, 158)', 'rgb(137, 117, 164)', 'rgb(116, 162, 164)'] },
  ],
  // Page 3 - Dark colors, single dots
  [
    { lightness: 10, algo: 'float', numDots: 1, position: '171,72', style: 'background:rgb(93, 86, 106);' },
    { lightness: 40, algo: 'float', numDots: 1, position: '265,79', style: 'background: #997096;' },
    { lightness: 35, algo: 'float', numDots: 1, position: '301,176', style: 'background: #956066;' },
    { lightness: 30, algo: 'float', numDots: 1, position: '237,210', style: 'background: #9c6645;' },
    { lightness: 30, algo: 'float', numDots: 1, position: '91,228', style: 'background: #517b6c;' },
    { lightness: 25, algo: 'float', numDots: 1, position: '67,159', style: 'background: #576e75;' },
    { lightness: 20, algo: 'float', numDots: 1, position: '314,235', style: 'background:rgb(131, 109, 95);' },
    { lightness: 20, algo: 'float', numDots: 1, position: '118,215', style: 'background: #447464;' },
  ],
  // Page 4 - Dark colors, analogous
  [
    { lightness: 10, algo: 'analogous', numDots: 3, position: '171,72', colors: ['rgb(23, 17, 34)', 'rgb(37, 14, 35)', 'rgb(18, 22, 33)'] },
    { lightness: 40, algo: 'analogous', numDots: 3, position: '265,79', colors: ['rgb(128, 76, 124)', 'rgb(141, 63, 66)', 'rgb(97, 88, 116)'] },
    { lightness: 35, algo: 'analogous', numDots: 3, position: '301,176', colors: ['rgb(122, 56, 64)', 'rgb(126, 121, 52)', 'rgb(111, 68, 110)'] },
    { lightness: 30, algo: 'analogous', numDots: 3, position: '237,210', colors: ['rgb(131, 65, 22)', 'rgb(64, 128, 25)', 'rgb(122, 31, 91)'] },
    { lightness: 30, algo: 'analogous', numDots: 3, position: '91,228', colors: ['rgb(45, 108, 85)', 'rgb(52, 85, 101)', 'rgb(52, 118, 35)'] },
    { lightness: 25, algo: 'analogous', numDots: 3, position: '67,159', colors: ['rgb(45, 74, 83)', 'rgb(46, 50, 81)', 'rgb(38, 90, 65)'] },
    { lightness: 20, algo: 'analogous', numDots: 3, position: '314,235', colors: ['rgb(64, 47, 38)', 'rgb(55, 64, 38)', 'rgb(59, 43, 52)'] },
    { lightness: 20, algo: 'analogous', numDots: 3, position: '118,215', colors: ['rgb(22, 80, 61)', 'rgb(26, 60, 76)', 'rgb(27, 87, 15)'] },
  ],
]