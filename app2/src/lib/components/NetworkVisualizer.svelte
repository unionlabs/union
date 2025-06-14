<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import { onMount } from "svelte"
import Card from "./ui/Card.svelte"

import type { TransferListItem } from "@unionlabs/sdk/schema"

// Extended transfer type with server pre-computed fields
type EnhancedTransferListItem = TransferListItem & {
  isTestnetTransfer?: boolean
  sourceDisplayName?: string
  destinationDisplayName?: string
  formattedTimestamp?: string
  routeKey?: string
  senderDisplay?: string
  receiverDisplay?: string
}

let {
  transfers = [],
  onChainSelection = () => {},
}: {
  transfers: EnhancedTransferListItem[]
  onChainSelection?: (fromChain: string | null, toChain: string | null) => void
} = $props()

// ===== COLOR CONFIGURATION =====
const COLOR_CONFIG = {
  chainDefault: "#e4e4e7", // Default chain node color (zinc-300)
  chainSelected: "#4bb7c3", // Selected chain color (accent blue)
  chainSelectedGlow: "#4bb7c3", // Selected chain glow color
  chainHit: "#ffffff", // Chain color when hit by transaction
  particle: "#fbbf24", // Transaction particle color (mainnet)
  particleTestnet: "#4bb7c3", // Transaction particle color for testnet (amber-400)
  particleGlow: "#ffffff", // Particle glow effect
  connectionDefault: "#52525b", // Normal connection lines (zinc-600)
  connectionSelected: "#4bb7c3", // Selected connection line (accent blue)
  uiBackground: "#000000cc", // UI box background (black with 80% alpha)
  uiText: "#ffffff", // UI text color
  uiTextSecondary: "#9ca3af", // Secondary UI text (gray-400)
  uiTextMuted: "#6b7280", // Muted UI text (gray-500)
}

// Constants
const PARTICLE_SPEED = 0.03
const TARGET_FPS = 120 // Reduced from 120 to 30 FPS - major CPU savings
const FRAME_INTERVAL = 1000 / TARGET_FPS
const MAX_PARTICLES = 200 // Limit particle count to prevent accumulation
const MOUSE_CHECK_INTERVAL = 5 // Check mouse hover every 5 frames instead of every frame

// Calculate responsive chain node size based on canvas dimensions
const getChainNodeSize = () => {
  const minSize = 5
  const maxSize = 18
  const baseSize = Math.min(canvasWidth, canvasHeight) * 0.02
  return Math.max(minSize, Math.min(maxSize, baseSize))
}

// Calculate responsive particle size based on canvas dimensions
const getParticleSize = () => {
  const minSize = 0.1
  const maxSize = 4
  const baseSize = Math.min(canvasWidth, canvasHeight) * 0.008
  return Math.max(minSize, Math.min(maxSize, baseSize))
}

let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let animationFrame: number
let lastFrameTime = 0
let canvasWidth = $state(800)
let canvasHeight = $state(600)
let containerElement: HTMLElement

// Mouse state
let mouseX = 0
let mouseY = 0
let hoveredChain: string | null = null

// Selection state
let selectedFromChain: string | null = $state(null)
let selectedToChain: string | null = $state(null)

// Animation state
let particles: Array<{
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
}> = []

let chainNodes: Map<string, {
  x: number
  y: number
  size: number
  pulseSize: number
  color: string
  activity: number
  displayName: string
  glowColor: string
  glowIntensity: number
}> = new Map()

// Performance optimization variables
let frameCount = 0
let particlePool: any[] = [] // Reuse particle objects

function updateCanvasSize() {
  if (!containerElement || !canvas || !ctx) {
    return
  }

  const rect = containerElement.getBoundingClientRect()
  canvasWidth = rect.width
  canvasHeight = rect.height

  const dpr = window.devicePixelRatio || 1
  canvas.width = canvasWidth * dpr
  canvas.height = canvasHeight * dpr
  canvas.style.width = canvasWidth + "px"
  canvas.style.height = canvasHeight + "px"

  if (ctx) {
    ctx.scale(dpr, dpr)
    ctx.imageSmoothingEnabled = false
  }

  // Recalculate chain positions when canvas resizes
  setupChainNodes()
}

function setupChainNodes() {
  if (!Option.isSome(chains.data)) {
    return
  }

  const chainData = chains.data.value
  if (!chainData || chainData.length === 0) {
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

    chainNodes.set(chain.universal_chain_id, {
      x,
      y,
      size: nodeSize,
      pulseSize: nodeSize,
      color: COLOR_CONFIG.chainDefault,
      activity: 0,
      displayName: chain.display_name || chain.chain_id,
      glowColor: COLOR_CONFIG.chainHit,
      glowIntensity: 0,
    })
  })
}

function createParticleFromTransfer(transfer: EnhancedTransferListItem) {
  // Limit particle count to prevent performance issues
  if (particles.length >= MAX_PARTICLES) {
    // Remove oldest particles if we're at the limit
    particles.splice(0, Math.floor(MAX_PARTICLES * 0.2)) // Remove 20% of oldest particles
  }

  if (
    !chainNodes.has(transfer.source_chain.universal_chain_id)
    || !chainNodes.has(transfer.destination_chain.universal_chain_id)
  ) {
    return
  }

  const fromNode = chainNodes.get(transfer.source_chain.universal_chain_id)!
  const toNode = chainNodes.get(transfer.destination_chain.universal_chain_id)!

  // Increase activity for both chains
  fromNode.activity = Math.min(fromNode.activity + 0.5, 3)
  toNode.activity = Math.min(toNode.activity + 0.5, 3)

  // Use pre-computed testnet flag from server
  const isTestnetTransfer = transfer.isTestnetTransfer || false

  // Reuse particle objects from pool or create new one
  let particle = particlePool.pop()
  if (!particle) {
    particle = {}
  }

  // Update particle properties
  Object.assign(particle, {
    id: transfer.packet_hash,
    x: fromNode.x,
    y: fromNode.y,
    startX: fromNode.x,
    startY: fromNode.y,
    targetX: toNode.x,
    targetY: toNode.y,
    fromChain: transfer.source_chain.universal_chain_id,
    toChain: transfer.destination_chain.universal_chain_id,
    value: parseFloat(transfer.base_amount.toString()) || 1,
    progress: 0,
    color: isTestnetTransfer ? COLOR_CONFIG.particleTestnet : COLOR_CONFIG.particle,
    size: getParticleSize(),
  })

  particles.push(particle)
}

function checkHover() {
  // Only check hover every few frames to reduce CPU usage
  if (frameCount % MOUSE_CHECK_INTERVAL !== 0) {
    return
  }

  hoveredChain = null
  chainNodes.forEach((node, chainId) => {
    const distance = Math.sqrt(
      (mouseX - node.x) * (mouseX - node.x)
        + (mouseY - node.y) * (mouseY - node.y),
    )
    if (distance <= node.size + 10) {
      hoveredChain = chainId
    }
  })

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

  // Optimize particle updates - use for loop instead of filter for better performance
  let activeParticles = []
  for (let i = 0; i < particles.length; i++) {
    const particle = particles[i]
    particle.progress += PARTICLE_SPEED

    if (particle.progress >= 1) {
      const toNode = chainNodes.get(particle.toChain)
      if (toNode) {
        toNode.pulseSize = toNode.size * 1.5
        toNode.glowColor = COLOR_CONFIG.chainHit
        toNode.glowIntensity = 1.0
      }
      // Return particle to pool for reuse
      particlePool.push(particle)
    } else {
      // Smooth interpolation - only calculate for active particles
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

  // Draw connection lines (optimized)
  if (chainNodes.size > 1) {
    const nodeArray = Array.from(chainNodes.entries())
    let selectedConnectionPairs: Array<[any, any]> = []

    // Set default connection style once
    ctx.strokeStyle = COLOR_CONFIG.connectionDefault
    ctx.lineWidth = 1
    ctx.beginPath()

    for (let i = 0; i < nodeArray.length; i++) {
      for (let j = i + 1; j < nodeArray.length; j++) {
        const [chainId1, node1] = nodeArray[i]
        const [chainId2, node2] = nodeArray[j]

        const isSelectedConnection =
          (selectedFromChain === chainId1 && selectedToChain === chainId2)
          || (selectedFromChain === chainId2 && selectedToChain === chainId1)

        if (isSelectedConnection) {
          selectedConnectionPairs.push([node1, node2])
        } else {
          ctx.moveTo(node1.x, node1.y)
          ctx.lineTo(node2.x, node2.y)
        }
      }
    }
    ctx.stroke()

    // Draw selected connections
    if (selectedConnectionPairs.length > 0) {
      ctx.strokeStyle = COLOR_CONFIG.connectionSelected
      ctx.lineWidth = 1
      ctx.globalAlpha = 0.6
      ctx.beginPath()
      selectedConnectionPairs.forEach(([node1, node2]) => {
        ctx.moveTo(node1.x, node1.y)
        ctx.lineTo(node2.x, node2.y)
      })
      ctx.stroke()
      ctx.globalAlpha = 1
    }
  }

  // Draw particles (optimized) - batch all particle drawing
  if (particles.length > 0) {
    ctx.fillStyle = COLOR_CONFIG.particle // Set once for mainnet particles
    particles.forEach(particle => {
      if (particle.color !== COLOR_CONFIG.particle) {
        ctx.fillStyle = particle.color // Only change if different
      }
      ctx.fillRect(
        particle.x - particle.size,
        particle.y - particle.size,
        particle.size * 2,
        particle.size * 2,
      )
    })
  }

  // Draw chain nodes (optimized) - batch similar operations
  chainNodes.forEach((node, chainId) => {
    // Update node state
    node.activity = Math.max(node.activity - 0.08, 0)
    node.glowIntensity = Math.max(node.glowIntensity - 0.05, 0)
  })

  // Batch draw all nodes
  chainNodes.forEach((node, chainId) => {
    const isSelected = chainId === selectedFromChain || chainId === selectedToChain
    const nodeRadius = node.size + (node.activity > 0 ? node.activity * 0.5 : 0)

    // Draw main node (simplified)
    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)

    // Simple color logic
    ctx.fillStyle = isSelected
      ? COLOR_CONFIG.chainSelected
      : node.glowIntensity > 0
      ? COLOR_CONFIG.chainHit
      : node.color
    ctx.fill()

    // Simple border for selected nodes only
    if (isSelected) {
      ctx.strokeStyle = COLOR_CONFIG.chainSelected
      ctx.lineWidth = 2
      ctx.stroke()
    }

    // Draw chain name on hover (no shadow) - only when actively hovering
    if (hoveredChain === chainId) {
      ctx.fillStyle = "rgba(255, 255, 255, 0.9)"
      ctx.font = "10px sans-serif"
      ctx.textAlign = "center"
      ctx.fillText(node.displayName, node.x, node.y - nodeRadius - 8)
    }
  })

  animationFrame = requestAnimationFrame(animate)
}

let processedTransferCount = $state(0)

// Process new transfers reactively
$effect(() => {
  if (transfers.length > processedTransferCount) {
    const newTransfers = transfers.slice(processedTransferCount)
    newTransfers.forEach(transfer => {
      createParticleFromTransfer(transfer)
    })
    processedTransferCount = transfers.length
  }
})

onMount(() => {
  if (canvas && containerElement) {
    ctx = canvas.getContext("2d")!

    // Initial canvas setup
    updateCanvasSize()

    canvas.addEventListener("mousemove", (e) => {
      const rect = canvas.getBoundingClientRect()
      mouseX = e.clientX - rect.left
      mouseY = e.clientY - rect.top
    })

    canvas.addEventListener("mouseleave", () => {
      hoveredChain = null
      canvas.style.cursor = "default"
    })

    canvas.addEventListener("click", handleChainClick)

    // Add resize observer to handle container size changes
    const resizeObserver = new ResizeObserver(() => {
      updateCanvasSize()
    })
    resizeObserver.observe(containerElement)

    animate()

    return () => {
      cancelAnimationFrame(animationFrame)
      resizeObserver.disconnect()
    }
  }

  return () => {
    cancelAnimationFrame(animationFrame)
  }
})

$effect(() => {
  if (Option.isSome(chains.data)) {
    setupChainNodes()
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
        style="background-color: {COLOR_CONFIG.uiBackground};"
      >
        <div
          class="text-xs mb-1"
          style="color: {COLOR_CONFIG.uiTextSecondary};"
        >
          Route Latency
        </div>
        <div
          class="text-sm font-medium mb-2"
          style="color: {COLOR_CONFIG.uiText};"
        >
          {chainNodes.get(selectedFromChain)?.displayName || selectedFromChain} →
          {chainNodes.get(selectedToChain)?.displayName || selectedToChain}
        </div>

        <div
          class="text-xs mt-2"
          style="color: {COLOR_CONFIG.uiTextMuted};"
        >
          Click elsewhere to clear selection
        </div>
      </div>
    {/if}
    <canvas
      bind:this={canvas}
      class="w-full h-full"
      style="background: transparent;"
    ></canvas>
  </div>
</Card>
