<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import { onMount } from "svelte"
import { initializeCanvasWithCleanup } from "../canvasInit"
import Card from "$lib/components/ui/Card.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"

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
  chainDefault: "#e4e4e7",
  chainSelected: "#4bb7c3", 
  chainHit: "#ffffff",
  particle: "#fbbf24",
  particleTestnet: "#4bb7c3",
  connectionDefault: "#52525b",
  connectionSelected: "#4bb7c3",
  uiBackground: "#000000cc",
  uiText: "#ffffff",
  uiTextSecondary: "#9ca3af",
  uiTextMuted: "#6b7280"
} as const

// Performance constants
const PARTICLE_SPEED = 0.03
const TARGET_FPS = 120
const FRAME_INTERVAL = 1000 / TARGET_FPS
const MAX_PARTICLES = 200
const MOUSE_CHECK_INTERVAL = 5

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
  if (!containerElement || !canvas || !ctx) return

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
  if (!Option.isSome(chains.data)) return

  const chainData = chains.data.value
  if (!chainData?.length) return

  chainNodes.clear()

  const centerX = canvasWidth / 2
  const centerY = canvasHeight / 2
  const radius = Math.min(canvasWidth, canvasHeight) * 0.45
  const nodeSize = getChainNodeSize()

  chainData.forEach((chain, index) => {
    const angle = (index / chainData.length) * 2 * Math.PI
    const x = centerX + Math.cos(angle) * radius
    const y = centerY + Math.sin(angle) * radius

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

  if (!chainNodes.has(sourceId) || !chainNodes.has(destId)) return

  const fromNode = chainNodes.get(sourceId)!
  const toNode = chainNodes.get(destId)!

  // Increase activity for both chains
  fromNode.activity = Math.min(fromNode.activity + 0.5, 3)
  toNode.activity = Math.min(toNode.activity + 0.5, 3)

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
  if (frameCount % MOUSE_CHECK_INTERVAL !== 0) return

  hoveredChain = null
  for (const [chainId, node] of chainNodes) {
    const distance = Math.sqrt(
      (mouseX - node.x) ** 2 + (mouseY - node.y) ** 2
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

      const isSelected = 
        (selectedFromChain === chainId1 && selectedToChain === chainId2) ||
        (selectedFromChain === chainId2 && selectedToChain === chainId1)

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
  if (particles.length === 0) return

  ctx.fillStyle = COLORS.particle
  for (const particle of particles) {
    if (particle.color !== COLORS.particle) {
      ctx.fillStyle = particle.color
    }
    ctx.fillRect(
      particle.x - particle.size,
      particle.y - particle.size,
      particle.size * 2,
      particle.size * 2
    )
  }
}

function updateAndDrawNodes() {
  // Update node states
  for (const node of chainNodes.values()) {
    node.activity = Math.max(node.activity - 0.08, 0)
    node.glowIntensity = Math.max(node.glowIntensity - 0.05, 0)
  }

  // Draw nodes
  for (const [chainId, node] of chainNodes) {
    const isSelected = chainId === selectedFromChain || chainId === selectedToChain
    const nodeRadius = node.size + (node.activity > 0 ? node.activity * 0.5 : 0)

    // Main node
    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)
    
    ctx.fillStyle = isSelected
      ? COLORS.chainSelected
      : node.glowIntensity > 0
      ? COLORS.chainHit
      : node.color
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
        { element: canvas, event: "mouseleave", handler: handleMouseLeave as (event: Event) => void },
        { element: canvas, event: "click", handler: handleChainClick as (event: Event) => void }
      ]
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
  <div class="relative w-full h-full" bind:this={containerElement}>
    {#if selectedFromChain && selectedToChain}
      <div
        class="absolute top-0 left-0 z-10 rounded-lg text-white min-w-48"
        style="background-color: {COLORS.uiBackground};"
      >
        <div class="text-xs mb-1" style="color: {COLORS.uiTextSecondary};">
          Route Latency
        </div>
        <div class="text-sm font-medium mb-2" style="color: {COLORS.uiText};">
          {chainNodes.get(selectedFromChain)?.displayName || selectedFromChain} →
          {chainNodes.get(selectedToChain)?.displayName || selectedToChain}
        </div>
        <div class="text-xs mt-2" style="color: {COLORS.uiTextMuted};">
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
