<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import { onMount } from "svelte"
import Card from "./ui/Card.svelte"

import type { TransferListItem } from "@unionlabs/sdk/schema"

let {
  transfers = [],
  onChainSelection = () => {},
}: {
  transfers: TransferListItem[]
  onChainSelection?: (fromChain: string | null, toChain: string | null) => void
} = $props()

// ===== COLOR CONFIGURATION =====
const COLOR_CONFIG = {
  chainDefault: "#e4e4e7", // Default chain node color (zinc-300)
  chainSelected: "#4bb7c3", // Selected chain color (accent blue)
  chainSelectedGlow: "#4bb7c3", // Selected chain glow color
  chainHit: "#ffffff", // Chain color when hit by transaction
  particle: "#fbbf24", // Transaction particle color (mainnet)
  particleTestnet: "#ffffff", // Transaction particle color for testnet (amber-400)
  particleGlow: "#ffffff", // Particle glow effect
  connectionDefault: "#52525b", // Normal connection lines (zinc-600)
  connectionSelected: "#4bb7c3", // Selected connection line (accent blue)
  uiBackground: "#000000cc", // UI box background (black with 80% alpha)
  uiText: "#ffffff", // UI text color
  uiTextSecondary: "#9ca3af", // Secondary UI text (gray-400)
  uiTextMuted: "#6b7280", // Muted UI text (gray-500)
}

// Constants
const CHAIN_BASE_SIZE = 12
const PARTICLE_SPEED = 0.02

let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let animationFrame: number
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
  const radius = Math.min(canvasWidth, canvasHeight) * 0.3

  chainData.forEach((chain, index) => {
    const angle = (index / chainData.length) * 2 * Math.PI
    const x = centerX + Math.cos(angle) * radius
    const y = centerY + Math.sin(angle) * radius

    chainNodes.set(chain.universal_chain_id, {
      x,
      y,
      size: CHAIN_BASE_SIZE,
      pulseSize: CHAIN_BASE_SIZE,
      color: COLOR_CONFIG.chainDefault,
      activity: 0,
      displayName: chain.display_name || chain.chain_id,
      glowColor: COLOR_CONFIG.chainHit,
      glowIntensity: 0,
    })
  })
}

function createParticleFromTransfer(transfer: TransferListItem) {
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

  // Check if either source or destination chain is testnet
  let isTestnetTransfer = false
  if (Option.isSome(chains.data)) {
    const chainData = chains.data.value
    const sourceChain = chainData.find(c =>
      c.universal_chain_id === transfer.source_chain.universal_chain_id
    )
    const destChain = chainData.find(c =>
      c.universal_chain_id === transfer.destination_chain.universal_chain_id
    )
    isTestnetTransfer = (sourceChain?.testnet || destChain?.testnet) || false
  }

  particles.push({
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
    size: 3,
  })
}

function checkHover() {
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

function animate() {
  if (!ctx) {
    return
  }

  checkHover()
  ctx.clearRect(0, 0, canvasWidth, canvasHeight)

  // Update particles
  particles = particles.filter(particle => {
    particle.progress += PARTICLE_SPEED

    if (particle.progress >= 1) {
      const toNode = chainNodes.get(particle.toChain)
      if (toNode) {
        toNode.pulseSize = toNode.size * 1.5
        toNode.glowColor = COLOR_CONFIG.chainHit
        toNode.glowIntensity = 1.0
      }
      return false
    }

    // Smooth interpolation
    const t = particle.progress
    const easedT = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
    particle.x = particle.startX + (particle.targetX - particle.startX) * easedT
    particle.y = particle.startY + (particle.targetY - particle.startY) * easedT

    return true
  })

  if (chainNodes.size === 0) {
    animationFrame = requestAnimationFrame(animate)
    return
  }

  // Draw connection lines
  const nodeArray = Array.from(chainNodes.entries())
  let selectedConnectionPairs: Array<[any, any]> = []

  for (let i = 0; i < nodeArray.length; i++) {
    for (let j = i + 1; j < nodeArray.length; j++) {
      const [chainId1, node1] = nodeArray[i]
      const [chainId2, node2] = nodeArray[j]

      const isSelectedConnection = (selectedFromChain === chainId1 && selectedToChain === chainId2)
        || (selectedFromChain === chainId2 && selectedToChain === chainId1)

      if (isSelectedConnection) {
        selectedConnectionPairs.push([node1, node2])
      } else {
        ctx.beginPath()
        ctx.moveTo(node1.x, node1.y)
        ctx.lineTo(node2.x, node2.y)
        ctx.strokeStyle = COLOR_CONFIG.connectionDefault
        ctx.lineWidth = 1
        ctx.stroke()
      }
    }
  }

  // Draw chain nodes
  chainNodes.forEach((node, chainId) => {
    // Update node state
    node.activity = Math.max(node.activity - 0.08, 0)
    node.glowIntensity = Math.max(node.glowIntensity - 0.05, 0)
    node.pulseSize = node.pulseSize + (node.size - node.pulseSize) * 0.1

    const nodeRadius = node.size + node.activity * 0.8
    const isSelected = chainId === selectedFromChain || chainId === selectedToChain

    // Draw pulse effect
    if (node.activity > 0) {
      const pulseRadius = node.size + node.activity * 2.5
      const pulseGradient = ctx.createRadialGradient(
        node.x,
        node.y,
        node.size,
        node.x,
        node.y,
        pulseRadius,
      )
      pulseGradient.addColorStop(0, node.color + "20")
      pulseGradient.addColorStop(1, node.color + "00")

      ctx.beginPath()
      ctx.arc(node.x, node.y, pulseRadius, 0, 2 * Math.PI)
      ctx.fillStyle = pulseGradient
      ctx.fill()
    }

    // Draw outer glow
    const outerGlow = ctx.createRadialGradient(
      node.x,
      node.y,
      0,
      node.x,
      node.y,
      nodeRadius * (isSelected ? 2.2 : 1.5),
    )
    const glowOpacity = isSelected ? "80" : "40"
    const glowColor = isSelected ? COLOR_CONFIG.chainSelectedGlow : node.color
    outerGlow.addColorStop(0, glowColor + glowOpacity)
    outerGlow.addColorStop(1, glowColor + "00")

    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius * (isSelected ? 2.2 : 1.5), 0, 2 * Math.PI)
    ctx.fillStyle = outerGlow
    ctx.fill()

    // Draw main node
    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)

    // Determine node fill color based on selection and glow state
    let fillColor = isSelected ? COLOR_CONFIG.chainSelected : node.color

    if (node.glowIntensity > 0) {
      const glowAmount = node.glowIntensity
      const baseAmount = 1 - glowAmount
      const r1 = parseInt(fillColor.slice(1, 3), 16)
      const g1 = parseInt(fillColor.slice(3, 5), 16)
      const b1 = parseInt(fillColor.slice(5, 7), 16)
      const r2 = parseInt(node.glowColor.slice(1, 3), 16)
      const g2 = parseInt(node.glowColor.slice(3, 5), 16)
      const b2 = parseInt(node.glowColor.slice(5, 7), 16)

      const r = Math.round(r1 * baseAmount + r2 * glowAmount)
      const g = Math.round(g1 * baseAmount + g2 * glowAmount)
      const b = Math.round(b1 * baseAmount + b2 * glowAmount)

      ctx.fillStyle = `rgb(${r}, ${g}, ${b})`
    } else {
      ctx.fillStyle = fillColor
    }
    ctx.fill()

    // Draw border
    ctx.beginPath()
    ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)
    if (isSelected) {
      ctx.strokeStyle = COLOR_CONFIG.chainSelected
      ctx.lineWidth = 2
      ctx.globalAlpha = 0.8
    } else {
      ctx.strokeStyle = "rgba(255, 255, 255, 0.2)"
      ctx.lineWidth = 1
      ctx.globalAlpha = 1
    }
    ctx.stroke()
    ctx.globalAlpha = 1

    // Draw chain name on hover
    if (hoveredChain === chainId) {
      ctx.fillStyle = "rgba(255, 255, 255, 0.9)"
      ctx.font = "10px -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif"
      ctx.textAlign = "center"
      ctx.shadowColor = "rgba(0, 0, 0, 0.7)"
      ctx.shadowBlur = 4
      ctx.shadowOffsetX = 0
      ctx.shadowOffsetY = 1
      ctx.fillText(node.displayName, node.x, node.y - nodeRadius - 8)
      ctx.shadowColor = "transparent"
      ctx.shadowBlur = 0
      ctx.shadowOffsetX = 0
      ctx.shadowOffsetY = 0
    }
  })

  // Draw selected connections
  selectedConnectionPairs.forEach(([node1, node2]) => {
    ctx.beginPath()
    ctx.moveTo(node1.x, node1.y)
    ctx.lineTo(node2.x, node2.y)
    ctx.strokeStyle = COLOR_CONFIG.connectionSelected
    ctx.lineWidth = 1
    ctx.globalAlpha = 0.6
    ctx.stroke()
  })
  ctx.globalAlpha = 1

  // Draw particles
  particles.forEach(particle => {
    // Outer glow
    const glowGradient = ctx.createRadialGradient(
      particle.x,
      particle.y,
      0,
      particle.x,
      particle.y,
      particle.size * 2.5,
    )
    glowGradient.addColorStop(0, COLOR_CONFIG.particleGlow)
    glowGradient.addColorStop(1, particle.color + "00")

    ctx.beginPath()
    ctx.arc(particle.x, particle.y, particle.size * 2.5, 0, 2 * Math.PI)
    ctx.fillStyle = glowGradient
    ctx.fill()

    // Main particle
    ctx.beginPath()
    ctx.arc(particle.x, particle.y, particle.size, 0, 2 * Math.PI)
    ctx.fillStyle = particle.color
    ctx.fill()
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

<Card class="h-full">
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
