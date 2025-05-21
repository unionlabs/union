/**
 * Checks if an image URL is valid by attempting to load it
 */
export async function isValidImageUrl(url: string): Promise<boolean> {
  if (!url) {
    return false
  }

  try {
    const response = await fetch(url, { method: "HEAD" })
    const contentType = response.headers.get("content-type")
    return response.ok && (contentType?.startsWith("image/") ?? false)
  } catch {
    return false
  }
}

/**
 * Generates a deterministic color based on a string
 */
function stringToColor(str: string): string {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash)
  }

  // Generate HSL color with fixed saturation and lightness for visibility
  const h = Math.abs(hash % 360)
  return `hsl(${h}, 70%, 60%)`
}

/**
 * Generates a data URL for an SVG avatar with crypto-aesthetic design
 */
export function generateAvatar(name: string): string {
  // Generate colors
  const color1 = stringToColor(name)
  const match = color1.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/)
  const hue = match ? (parseInt(match[1]) + 120) % 360 : 0
  const color2 = `hsl(${hue}, 100%, 60%)`
  const color3 = `hsl(${(hue + 120) % 360}, 100%, 60%)`

  // Generate unique pattern based on name
  const hash = name.split("").reduce(
    (acc, char) => char.charCodeAt(0) + acc,
    0,
  )
  const numShapes = (hash % 4) + 4 // 4-7 shapes

  let shapes = ""
  for (let i = 0; i < numShapes; i++) {
    const shapeType = (hash + i) % 3 // 0: rectangle, 1: square, 2: glitch rectangle
    const x = ((hash * (i + 1)) % 30) + 5
    const y = ((hash * (i + 2)) % 30) + 5
    const width = shapeType === 0 ? 12 : 8
    const height = shapeType === 0 ? 6 : 8
    const color = i % 3 === 0 ? color1 : i % 3 === 1 ? color2 : color3

    if (shapeType === 2) {
      // Glitch effect with multiple offset rectangles
      const glitchOffset = 2
      shapes += `
        <rect 
          x="${x}" y="${y}" 
          width="${width}" height="${height}" 
          fill="${color}"
          opacity="0.8"
        >
          <animate
            attributeName="x"
            values="${x};${x + glitchOffset};${x}"
            dur="${0.1 + (i * 0.1)}s"
            repeatCount="indefinite"
            begin="${i * 0.2}s"
          />
        </rect>
        <rect 
          x="${x - glitchOffset}" y="${y}" 
          width="${width}" height="${height}" 
          fill="${color}"
          opacity="0.5"
        >
          <animate
            attributeName="x"
            values="${x - glitchOffset};${x};${x - glitchOffset}"
            dur="${0.2 + (i * 0.1)}s"
            repeatCount="indefinite"
            begin="${i * 0.1}s"
          />
        </rect>
      `
    } else {
      shapes += `
        <rect 
          x="${x}" y="${y}" 
          width="${width}" height="${height}" 
          fill="${color}"
        >
          <animate
            attributeName="opacity"
            values="1;0.7;1"
            dur="${1 + (i * 0.5)}s"
            repeatCount="indefinite"
            begin="${i * 0.2}s"
          />
        </rect>
      `
    }
  }

  const svg = `
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40">
      <rect width="40" height="40" fill="#111" />
      ${shapes}
    </svg>
  `

  return `data:image/svg+xml;base64,${btoa(svg)}`
}
