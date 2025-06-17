<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import { chains } from "$lib/stores/chains.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { onMount } from "svelte"
import { initializeCanvasWithCleanup } from "../canvasInit"

type EnhancedTransferListItem = TransferListItem & {
  isTestnetTransfer?: boolean
  formattedTimestamp?: string
  routeKey?: string
  senderDisplay?: string
  receiverDisplay?: string
}

interface Particle {
  id: string
  x: number
  y: number
  startX: number
  startY: number
  targetX: number
  targetY: number
  fromChain: string
  toChain: string
  value: number
  progress: number
  color: string
  size: number
}

interface ChainNode {
  x: number
  y: number
  size: number
  pulseSize: number
  color: string
  activity: number
  displayName: string
  glowColor: string
  glowIntensity: number
  lastActivity: number // Timestamp of last activity
  fadeLevel: number // 0-1, how faded the node is (0 = fully faded, 1 = fully bright)
}

let {
  transfers = [],
  onChainSelection = () => {},
}: {
  transfers: EnhancedTransferListItem[]
  onChainSelection?: (fromChain: string | null, toChain: string | null) => void
} = $props()

// Color configuration
const COLORS = {
  chainActive: "#ffffff", // Bright white for active chains
  chainDefault: "#e4e4e7", // Normal state
  chainFaded: "#52525b", // Faded inactive chains
  chainSelected: "#4bb7c3",
  chainHit: "#ffffff",
  particle: "#fbbf24",
  particleTestnet: "#4bb7c3",
  connectionDefault: "#52525b",
  connectionSelected: "#4bb7c3",
  uiBackground: "#000000cc",
  uiText: "#ffffff",
  uiTextSecondary: "#9ca3af",
  uiTextMuted: "#6b7280",
} as const

// Performance constants
const PARTICLE_SPEED = 0.03
const TARGET_FPS = 120
const FRAME_INTERVAL = 1000 / TARGET_FPS
const MAX_PARTICLES = 200
const MOUSE_CHECK_INTERVAL = 5

// Fade system constants
const FADE_DELAY = 3000 // Start fading after 3 seconds of inactivity
const FADE_DURATION = 5000 // Take 5 seconds to fully fade
const MIN_FADE_LEVEL = 0.2 // Don't fade completely (30% minimum brightness)

// Responsive sizing functions
const getChainNodeSize = () => {
  const baseSize = Math.min(canvasWidth, canvasHeight) * 0.02
  return Math.max(5, Math.min(18, baseSize))
}

const getParticleSize = () => {
  const baseSize = Math.min(canvasWidth, canvasHeight) * 0.008
  return Math.max(0.1, Math.min(4, baseSize))
}

// State variables
let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let animationFrame: number
let lastFrameTime = 0
let canvasWidth = $state(800)
let canvasHeight = $state(600)
let containerElement: HTMLElement
let canvasReady = $state(false)

// Mouse state
let mouseX = 0
let mouseY = 0
let hoveredChain: string | null = null

// Selection state
let selectedFromChain: string | null = $state(null)
let selectedToChain: string | null = $state(null)

// Animation state
let particles: Particle[] = []
let chainNodes = new Map<string, ChainNode>()
let frameCount = 0
let particlePool: Particle[] = []

// Reactive derived values
let transfersLength = $derived(transfers.length)
let processedTransferCount = $state(0)

// Canvas functions
function updateCanvasSize() {
  if (!containerElement || !canvas || !ctx) {
    return
  }

  const rect = containerElement.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  canvasWidth = rect.width
  canvasHeight = rect.height

  canvas.width = canvasWidth * dpr
  canvas.height = canvasHeight * dpr
  canvas.style.width = `${canvasWidth}px`
  canvas.style.height = `${canvasHeight}px`

  if (ctx) {
    ctx.scale(dpr, dpr)
    ctx.imageSmoothingEnabled = false
  }

  setupChainNodes()
}

function setupChainNodes() {
  if (!Option.isSome(chains.data)) {
    return
  }

  const chainData = chains.data.value
  if (!chainData?.length) {
    return
  }

  chainNodes.clear()

  const centerX = canvasWidth / 2
  const centerY = canvasHeight / 2
  const radius = Math.min(canvasWidth, canvasHeight) * 0.45
  const nodeSize = getChainNodeSize()

  chainData.forEach((chain, index) => {
    const angle = (index / chainData.length) * 2 * Math.PI
    const x = centerX + Math.cos(angle) * radius
    const y = centerY + Math.sin(angle) * radius

    const now = Date.now()
    // Start nodes in faded state since they haven't had activity yet
    const initialLastActivity = now - (FADE_DELAY + FADE_DURATION)

    chainNodes.set(chain.universal_chain_id, {
      x,
      y,
      size: nodeSize,
      pulseSize: nodeSize,
      color: COLORS.chainDefault,
      activity: 0,
      displayName: chain.display_name || chain.chain_id,
      glowColor: COLORS.chainHit,
      glowIntensity: 0,
      lastActivity: initialLastActivity,
      fadeLevel: MIN_FADE_LEVEL, // Start at minimum fade level
    })
  })
}

function createParticleFromTransfer(transfer: EnhancedTransferListItem) {
  // Particle count optimization
  if (particles.length >= MAX_PARTICLES) {
    particles.splice(0, Math.floor(MAX_PARTICLES * 0.2))
  }

  const sourceId = transfer.source_chain.universal_chain_id
  const destId = transfer.destination_chain.universal_chain_id

  if (!chainNodes.has(sourceId) || !chainNodes.has(destId)) {
    return
  }

  const fromNode = chainNodes.get(sourceId)!
  const toNode = chainNodes.get(destId)!

  // Increase activity for both chains and reset fade
  const now = Date.now()
  fromNode.activity = Math.min(fromNode.activity + 0.5, 3)
  fromNode.lastActivity = now
  fromNode.fadeLevel = 1 // Reset to full brightness

  toNode.activity = Math.min(toNode.activity + 0.5, 3)
  toNode.lastActivity = now
  toNode.fadeLevel = 1 // Reset to full brightness

  // Reuse particle objects from pool
  let particle = particlePool.pop() || {} as Particle

  Object.assign(particle, {
    id: transfer.packet_hash,
    x: fromNode.x,
    y: fromNode.y,
    startX: fromNode.x,
    startY: fromNode.y,
    targetX: toNode.x,
    targetY: toNode.y,
    fromChain: sourceId,
    toChain: destId,
    value: 1,
    progress: 0,
    color: transfer.isTestnetTransfer ? COLORS.particleTestnet : COLORS.particle,
    size: getParticleSize(),
  })

  particles.push(particle)
}

function checkHover() {
  if (frameCount % MOUSE_CHECK_INTERVAL !== 0) {
    return
  }

  hoveredChain = null
  for (const [chainId, node] of chainNodes) {
    const distance = Math.sqrt(
      (mouseX - node.x) ** 2 + (mouseY - node.y) ** 2,
    )
    if (distance <= node.size + 10) {
      hoveredChain = chainId
      break
    }
  }

  if (canvas) {
    canvas.style.cursor = hoveredChain ? "pointer" : "default"
  }
}

function handleChainClick() {
  if (!hoveredChain) {
    selectedFromChain = null
    selectedToChain = null
    onChainSelection(null, null)
    return
  }

  if (!selectedFromChain) {
    selectedFromChain = hoveredChain
    selectedToChain = null
    onChainSelection(hoveredChain, null)
  } else if (!selectedToChain && hoveredChain !== selectedFromChain) {
    selectedToChain = hoveredChain
    onChainSelection(selectedFromChain, hoveredChain)
  } else {
    selectedFromChain = hoveredChain
    selectedToChain = null
    onChainSelection(hoveredChain, null)
  }
}

function animate(currentTime = 0) {
  if (!ctx) {
    animationFrame = requestAnimationFrame(animate)
    return
  }

  // Frame rate limiting
  if (currentTime - lastFrameTime < FRAME_INTERVAL) {
    animationFrame = requestAnimationFrame(animate)
    return
  }
  lastFrameTime = currentTime
  frameCount++

  checkHover()
  ctx.clearRect(0, 0, canvasWidth, canvasHeight)

  // Update particles
  const activeParticles: Particle[] = []
  for (const particle of particles) {
    particle.progress += PARTICLE_SPEED

    if (particle.progress >= 1) {
      const toNode = chainNodes.get(particle.toChain)
      if (toNode) {
        toNode.pulseSize = toNode.size * 1.5
        toNode.glowColor = COLORS.chainHit
        toNode.glowIntensity = 1.0
        toNode.lastActivity = Date.now()
        toNode.fadeLevel = 1 // Reset to full brightness on hit
      }
      particlePool.push(particle)
    } else {
      // Smooth easing
      const t = particle.progress
      const easedT = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
      particle.x = particle.startX + (particle.targetX - particle.startX) * easedT
      particle.y = particle.startY + (particle.targetY - particle.startY) * easedT
      activeParticles.push(particle)
    }
  }
  particles = activeParticles

  if (chainNodes.size === 0) {
    animationFrame = requestAnimationFrame(animate)
    return
  }

  // Draw connection lines
  if (chainNodes.size > 1) {
    drawConnections()
  }

  // Draw particles
  drawParticles()

  // Update and draw chain nodes
  updateAndDrawNodes()

  animationFrame = requestAnimationFrame(animate)
}

function drawConnections() {
  const nodeArray = Array.from(chainNodes.entries())
  const selectedConnections: Array<[ChainNode, ChainNode]> = []

  // Default connections
  ctx.strokeStyle = COLORS.connectionDefault
  ctx.lineWidth = 1
  ctx.beginPath()

  for (let i = 0; i < nodeArray.length; i++) {
    for (let j = i + 1; j < nodeArray.length; j++) {
      const [chainId1, node1] = nodeArray[i]
      const [chainId2, node2] = nodeArray[j]

      const isSelected = (selectedFromChain === chainId1 && selectedToChain === chainId2)
        || (selectedFromChain === chainId2 && selectedToChain === chainId1)

      if (isSelected) {
        selectedConnections.push([node1, node2])
      } else {
        ctx.moveTo(node1.x, node1.y)
        ctx.lineTo(node2.x, node2.y)
      }
    }
  }
  ctx.stroke()

  // Selected connections
  if (selectedConnections.length > 0) {
    ctx.strokeStyle = COLORS.connectionSelected
    ctx.lineWidth = 1
    ctx.globalAlpha = 0.6
    ctx.beginPath()
    for (const [node1, node2] of selectedConnections) {
      ctx.moveTo(node1.x, node1.y)
      ctx.lineTo(node2.x, node2.y)
    }
    ctx.stroke()
    ctx.globalAlpha = 1
  }
}

function drawParticles() {
  if (particles.length === 0) {
    return
  }

  ctx.fillStyle = COLORS.particle
  for (const particle of particles) {
    if (particle.color !== COLORS.particle) {
      ctx.fillStyle = particle.color
    }
    ctx.fillRect(
      particle.x - particle.size,
      particle.y - particle.size,
      particle.size * 2,
      particle.size * 2,
    )
  }
}

function updateAndDrawNodes() {
  const now = Date.now()

  // Update node states
  for (const node of chainNodes.values()) {
    node.activity = Math.max(node.activity - 0.08, 0)
    node.glowIntensity = Math.max(node.glowIntensity - 0.05, 0)

    // Calculate fade level based on time since last activity
    const timeSinceActivity = now - node.lastActivity
    if (timeSinceActivity > FADE_DELAY) {
      const fadeProgress = Math.min((timeSinceActivity - FADE_DELAY) / FADE_DURATION, 1)
      node.fadeLevel = 1 - (fadeProgress * (1 - MIN_FADE_LEVEL))
    } else {
      node.fadeLevel = 1 // No fading during delay period
    }
  }

  // Draw nodes
  for (const [chainId, node] of chainNodes) {
    const isSelected = chainId === selectedFromChain || chainId === selectedToChain
    const isHovered = hoveredChain === chainId
    const nodeRadius = node.size + (node.activity > 0 ? node.activity * 0.5 : 0)

    // Keep selected/hovered nodes bright
    if (isSelected || isHovered) {
      node.fadeLevel = Math.max(node.fadeLevel, 0.8)
    }

    // Main node
    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)

    let nodeColor: string
    if (isSelected) {
      nodeColor = COLORS.chainSelected
    } else if (node.glowIntensity > 0) {
      nodeColor = COLORS.chainHit
    } else {
      // Interpolate between faded and normal/active based on fade level
      const baseColor = node.activity > 0 ? COLORS.chainActive : COLORS.chainDefault
      if (node.fadeLevel < 1) {
        // Create interpolated color between base and faded
        const r1 = parseInt(baseColor.slice(1, 3), 16)
        const g1 = parseInt(baseColor.slice(3, 5), 16)
        const b1 = parseInt(baseColor.slice(5, 7), 16)

        const r2 = parseInt(COLORS.chainFaded.slice(1, 3), 16)
        const g2 = parseInt(COLORS.chainFaded.slice(3, 5), 16)
        const b2 = parseInt(COLORS.chainFaded.slice(5, 7), 16)

        const r = Math.round(r2 + (r1 - r2) * node.fadeLevel)
        const g = Math.round(g2 + (g1 - g2) * node.fadeLevel)
        const b = Math.round(b2 + (b1 - b2) * node.fadeLevel)

        nodeColor = `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${
          b.toString(16).padStart(2, "0")
        }`
      } else {
        nodeColor = baseColor
      }
    }

    ctx.fillStyle = nodeColor
    ctx.fill()

    // Border for selected nodes
    if (isSelected) {
      ctx.strokeStyle = COLORS.chainSelected
      ctx.lineWidth = 2
      ctx.stroke()
    }

    // Chain name on hover
    if (hoveredChain === chainId) {
      ctx.fillStyle = "rgba(255, 255, 255, 0.9)"
      ctx.font = "10px sans-serif"
      ctx.textAlign = "center"
      ctx.fillText(node.displayName, node.x, node.y - nodeRadius - 8)
    }
  }
}

// Event handlers
const handleMouseMove = (e: MouseEvent) => {
  const rect = canvas.getBoundingClientRect()
  mouseX = e.clientX - rect.left
  mouseY = e.clientY - rect.top
}

const handleMouseLeave = () => {
  hoveredChain = null
  canvas.style.cursor = "default"
}

// Process new transfers
$effect(() => {
  if (canvasReady && transfersLength > processedTransferCount) {
    const newTransfers = transfers.slice(processedTransferCount)
    newTransfers.forEach(transfer => {
      createParticleFromTransfer(transfer)
    })
    processedTransferCount = transfers.length
  }
})

// Setup chains when data is available
$effect(() => {
  if (Option.isSome(chains.data)) {
    setupChainNodes()
  }
})

onMount(() => {
  if (canvas && containerElement) {
    ctx = canvas.getContext("2d")!

    const cleanup = initializeCanvasWithCleanup({
      canvas,
      container: containerElement,
      onInitialized: () => {
        updateCanvasSize()
        canvasReady = true
        animate()
      },
      onResize: () => {
        updateCanvasSize()
      },
      eventListeners: [
        { element: canvas, event: "mousemove", handler: handleMouseMove as (event: Event) => void },
        {
          element: canvas,
          event: "mouseleave",
          handler: handleMouseLeave as (event: Event) => void,
        },
        { element: canvas, event: "click", handler: handleChainClick as (event: Event) => void },
      ],
    })

    return () => {
      cancelAnimationFrame(animationFrame)
      cleanup()
    }
  }

  return () => {
    cancelAnimationFrame(animationFrame)
  }
})
</script>

<Card class="h-full p-3">
  <div
    class="relative w-full h-full"
    bind:this={containerElement}
  >
    {#if selectedFromChain && selectedToChain}
      <div
        class="absolute top-0 left-0 z-10 rounded-lg text-white min-w-48"
        style="background-color: {COLORS.uiBackground};"
      >
        <div
          class="text-xs mb-1"
          style="color: {COLORS.uiTextSecondary};"
        >
          Route Latency
        </div>
        <div
          class="text-sm font-medium mb-2"
          style="color: {COLORS.uiText};"
        >
          {chainNodes.get(selectedFromChain)?.displayName || selectedFromChain} →
          {chainNodes.get(selectedToChain)?.displayName || selectedToChain}
        </div>
        <div
          class="text-xs mt-2"
          style="color: {COLORS.uiTextMuted};"
        >
          Click elsewhere to clear selection
        </div>
      </div>
    {/if}
    <canvas
      bind:this={canvas}
      class="w-full h-full"
      style="background: transparent;"
    />
  </div>
</Card>
