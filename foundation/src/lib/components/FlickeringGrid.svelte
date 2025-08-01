<script lang="ts">
let {
  squareSize = 4,
  gridGap = 6,
  flickerChance = 0.3,
  color = "rgb(0, 0, 0)",
  width,
  height,
  maxOpacity = 0.3,
  fadeTop = false,
  class: className = "",
}: {
  squareSize?: number
  gridGap?: number
  flickerChance?: number
  color?: string
  width?: number
  height?: number
  maxOpacity?: number
  fadeTop?: boolean
  class?: string
} = $props()

let canvas: HTMLCanvasElement
let isInView = $state(false)
let animationFrameId: number
let arrows: Array<{ x: number; y: number; opacity: number; speed: number; size: number }> = []

let memoizedColor: string

function toRGBA(color: string) {
  if (typeof window === "undefined") {
    return `rgba(0, 0, 0,`
  }
  const canvas = document.createElement("canvas")
  canvas.width = canvas.height = 1
  const ctx = canvas.getContext("2d")
  if (!ctx) {
    return "rgba(255, 0, 0,"
  }
  ctx.fillStyle = color
  ctx.fillRect(0, 0, 1, 1)
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data
  return `rgba(${r}, ${g}, ${b},`
}

memoizedColor = toRGBA(color)

function setupCanvas() {
  const canvasWidth = width || window.innerWidth
  const canvasHeight = height || window.innerHeight
  const dpr = window.devicePixelRatio || 1
  canvas.width = canvasWidth * dpr
  canvas.height = canvasHeight * dpr
  canvas.style.width = `${canvasWidth}px`
  canvas.style.height = `${canvasHeight}px`

  // Initialize arrows array
  arrows = []
  const cols = Math.ceil(canvasWidth / (squareSize + gridGap))
  const rows = Math.ceil(canvasHeight / (squareSize + gridGap))

  // Pre-populate screen with arrows at various heights
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      if (Math.random() < 0.055) { // Reduced by 30% from 8% to 5.5%
        const xOffset = ((i + j) % 3) * ((squareSize + gridGap) / 3)
        const yOffset = ((i * 2 + j) % 2) * ((squareSize + gridGap) / 4)

        arrows.push({
          x: i * (squareSize + gridGap) + xOffset,
          y: j * (squareSize + gridGap) + yOffset,
          opacity: Math.random() * maxOpacity,
          speed: 0.5 + Math.random() * 1.5,
          size: squareSize * (0.8 + Math.random() * 0.4), // Size variation: 80% to 120% of base size
        })
      }
    }
  }

  return { canvasWidth, canvasHeight, dpr }
}

function updateArrows(deltaTime: number, canvasWidth: number, canvasHeight: number) {
  const speedMultiplier = deltaTime * 60 // Normalize to 60fps

  // Move existing arrows upward
  for (let i = arrows.length - 1; i >= 0; i--) {
    arrows[i].y -= arrows[i].speed * speedMultiplier

    // Remove arrows that are off screen
    if (arrows[i].y < -arrows[i].size * 2) {
      arrows.splice(i, 1)
    }
  }

  // Spawn new arrows at bottom
  const cols = Math.ceil(canvasWidth / (squareSize + gridGap))
  for (let i = 0; i < cols; i++) {
    if (Math.random() < flickerChance * deltaTime * 3) { // Increased spawn rate by 3x
      const xOffset = ((i + Math.random()) % 3) * ((squareSize + gridGap) / 3)
      arrows.push({
        x: i * (squareSize + gridGap) + xOffset,
        y: canvasHeight + squareSize,
        opacity: Math.random() * maxOpacity,
        speed: 0.5 + Math.random() * 1.5,
        size: squareSize * (0.8 + Math.random() * 0.4), // Size variation: 80% to 120% of base size
      })
    }
  }
}

function drawArrows(
  ctx: CanvasRenderingContext2D,
  canvasWidth: number,
  canvasHeight: number,
  dpr: number,
) {
  ctx.clearRect(0, 0, canvasWidth * dpr, canvasHeight * dpr)

  for (const arrow of arrows) {
    let opacity = arrow.opacity

    // Apply fade effect from bottom to top if enabled
    if (fadeTop) {
      const fadeRatio = 1 - (arrow.y / canvasHeight) // 0 at bottom, 1 at top
      const aggressiveFade = Math.max(0, 1 - fadeRatio * fadeRatio * fadeRatio)
      opacity *= aggressiveFade
    }

    if (opacity <= 0) {
      continue
    }

    ctx.fillStyle = `${memoizedColor}${opacity})`

    const x = arrow.x * dpr
    const y = arrow.y * dpr
    const size = arrow.size * dpr
    const shaftExtension = 2 * dpr

    // Draw upward-pointing arrow
    ctx.beginPath()

    // Arrow head (triangle)
    const headHeight = size * 0.6
    const headWidth = size

    ctx.moveTo(x + size / 2, y) // Top point
    ctx.lineTo(x + (size - headWidth) / 2, y + headHeight) // Bottom left of head
    ctx.lineTo(x + size / 4 + headWidth / 8, y + headHeight) // Left side of shaft

    // Arrow shaft (extended by 2px)
    const shaftWidth = size * 0.3
    ctx.lineTo(x + (size - shaftWidth) / 2, y + headHeight) // Left side of shaft
    ctx.lineTo(x + (size - shaftWidth) / 2, y + size + shaftExtension) // Bottom left of shaft (extended)
    ctx.lineTo(x + (size + shaftWidth) / 2, y + size + shaftExtension) // Bottom right of shaft (extended)
    ctx.lineTo(x + (size + shaftWidth) / 2, y + headHeight) // Right side of shaft

    ctx.lineTo(x + size * 3 / 4 - headWidth / 8, y + headHeight) // Right side of shaft connection
    ctx.lineTo(x + (size + headWidth) / 2, y + headHeight) // Bottom right of head

    ctx.closePath()
    ctx.fill()
  }
}

$effect(() => {
  if (!canvas) {
    return
  }

  const ctx = canvas.getContext("2d")
  if (!ctx) {
    return
  }

  let { canvasWidth, canvasHeight, dpr } = setupCanvas()
  let lastTime = 0

  const animate = (time: number) => {
    if (!isInView) {
      return
    }
    const deltaTime = Math.min((time - lastTime) / 1000, 0.016) // Cap at 60fps
    lastTime = time

    updateArrows(deltaTime, canvasWidth, canvasHeight)
    drawArrows(
      ctx,
      canvasWidth * dpr,
      canvasHeight * dpr,
      dpr,
    )
    animationFrameId = requestAnimationFrame(animate)
  }

  const handleResize = () => {
    ;({ canvasWidth, canvasHeight, dpr } = setupCanvas())
  }

  const observer = new IntersectionObserver(
    ([entry]) => {
      isInView = entry.isIntersecting
      if (isInView && lastTime === 0) {
        lastTime = performance.now()
        animationFrameId = requestAnimationFrame(animate)
      }
    },
    { threshold: 0 },
  )

  observer.observe(canvas)
  window.addEventListener("resize", handleResize)

  // Start animation when in view
  if (isInView) {
    lastTime = performance.now()
    animationFrameId = requestAnimationFrame(animate)
  }

  return () => {
    window.removeEventListener("resize", handleResize)
    cancelAnimationFrame(animationFrameId)
    observer.disconnect()
  }
})
</script>

<canvas
  bind:this={canvas}
  class="absolute inset-0 w-full h-full pointer-events-none {className}"
  style="width: 100vw; height: 100vh; left: 0; top: 0;"
></canvas>
