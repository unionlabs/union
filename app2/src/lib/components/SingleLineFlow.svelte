<script lang="ts">
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { onMount } from "svelte"

interface Props {
  transfers: TransferListItem[]
  class?: string
}

let { transfers, class: className = "h-36" }: Props = $props()

let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let animationId: number
let particles: Particle[] = []

interface Particle {
  id: string
  x: number
  y: number
  speed: number
  size: number
  opacity: number
  life: number
  maxLife: number
}

const COLORS = {
  background: "transparent",
  line: "#71717a", // zinc-500
  particle: "#4bb7c3",
}

function createParticleFromTransfer(transfer: TransferListItem) {
  const particle: Particle = {
    id: transfer.packet_hash,
    x: -1,
    y: 0, // Not used since pixel is always on the line
    speed: 1 + Math.random() * 2,
    size: 4, // Always 1 pixel
    opacity: 1, // Always full opacity
    life: canvas.width + 10, // Live until off screen
    maxLife: canvas.width + 10,
  }

  particles.push(particle)
}

function drawLine() {
  const lineY = Math.floor(canvas.height * 0.5)

  // Draw zinc pixel line
  ctx.fillStyle = COLORS.line
  ctx.fillRect(0, lineY, canvas.width, 4)
}

function drawParticle(particle: Particle) {
  const lineY = Math.floor(canvas.height * 0.5)

  // Draw white pixel using the particle size
  ctx.fillStyle = COLORS.particle
  ctx.fillRect(Math.floor(particle.x), lineY, particle.size, particle.size)
}

function animate() {
  if (!ctx) {
    return
  }

  // Clear canvas
  ctx.fillStyle = COLORS.background
  ctx.fillRect(0, 0, canvas.width, canvas.height)

  // Draw the line
  drawLine()

  // Update and draw particles
  particles = particles.filter(particle => {
    particle.x += particle.speed

    if (particle.x <= canvas.width) {
      drawParticle(particle)
      return true
    }
    return false
  })

  animationId = requestAnimationFrame(animate)
}

function resizeCanvas() {
  if (!canvas) {
    return
  }

  const container = canvas.parentElement
  if (container) {
    canvas.width = container.clientWidth
    canvas.height = container.clientHeight
  }
}

// Watch for new transfers
$effect(() => {
  if (transfers.length > 0) {
    const latestTransfer = transfers[transfers.length - 1]
    createParticleFromTransfer(latestTransfer)
  }
})

onMount(() => {
  ctx = canvas.getContext("2d")!
  resizeCanvas()
  animate()

  const resizeObserver = new ResizeObserver(resizeCanvas)
  resizeObserver.observe(canvas.parentElement!)

  return () => {
    if (animationId) {
      cancelAnimationFrame(animationId)
    }
    resizeObserver.disconnect()
  }
})
</script>

<div class="w-full h-12 overflow-hidden{className}">
  <canvas
    bind:this={canvas}
    class="single-line-canvas"
  />
</div>

<style>
.single-line-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
