export const pathConfigMap = {
  "/": {
    index: 1,
    colors: {
      primary: hexToWebGL("#7ce9ff"),
      mid: hexToWebGL("#D0F7FF"),
      dark: hexToWebGL("#025C70")
    }
  },
  "/learn": {
    index: 2,
    colors: {
      primary: hexToWebGL("#0B1A40"),
      mid: hexToWebGL("#030711"),
      dark: hexToWebGL("#000102")
    }
  },
  "/ecosystem": {
    index: 3,
    colors: {
      primary: hexToWebGL("#3D3D3D"),
      mid: hexToWebGL("#151515"),
      dark: hexToWebGL("#2A2A2A")
    }
  },
  "/blog": {
    index: 4,
    colors: {
      primary: hexToWebGL("#66919A"),
      mid: hexToWebGL("#4E737B"),
      dark: hexToWebGL("#2E4449")
    }
  },
  "/team": {
    index: 5,
    colors: {
      primary: hexToWebGL("#FAF7F7"),
      mid: hexToWebGL("#B7B7B7"),
      dark: hexToWebGL("#7A7A7A")
    }
  }
} as const

function hexToWebGL(hex: string) {
  hex = hex.replace("#", "")

  const r = Number.parseInt(hex.substring(0, 2), 16) / 255
  const g = Number.parseInt(hex.substring(2, 4), 16) / 255
  const b = Number.parseInt(hex.substring(4, 6), 16) / 255

  return [r, g, b, 1.0]
}
