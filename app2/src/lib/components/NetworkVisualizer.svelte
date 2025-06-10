<script lang="ts">
  import { onMount } from 'svelte'
  import { transferList } from '$lib/stores/transfers.svelte'
  import { chains } from '$lib/stores/chains.svelte'
  import { transferListLatestQuery } from '$lib/queries/transfer-list.svelte'

  import { Option } from 'effect'
  import Sections from './ui/Sections.svelte';
  import Card from './ui/Card.svelte';

  // ===== COLOR CONFIGURATION =====
  const COLOR_CONFIG = {
    // Chain node colors
    chainDefault: '#e4e4e7',          // Default chain node color (zinc-300)
    chainSelected: '#ffffff',         // Selected chain glow color
    chainHit: '#ffffff',              // Chain color when hit by transaction
    
    // Particle colors  
    particle: '#ffffff',              // Transaction particle color
    particleGlow: '#ffffff',          // Particle glow effect
    
    // Connection line colors
    connectionDefault: '#52525b',     // Normal connection lines (zinc-600)
    connectionSelected: '#ffffff',    // Selected connection line
    
    // UI colors
    loadingSpinner: '#fff',           // Loading spinner color
    uiBackground: '#000000cc',        // UI box background (black with 80% alpha)
    uiText: '#ffffff',                // UI text color
    uiTextSecondary: '#9ca3af',       // Secondary UI text (gray-400)
    uiTextMuted: '#6b7280'            // Muted UI text (gray-500)
  }

  let canvas: HTMLCanvasElement
  let ctx: CanvasRenderingContext2D
  let animationFrame: number
  let canvasWidth = 800
  let canvasHeight = 600
  
  // Mouse state for hover detection
  let mouseX = 0
  let mouseY = 0
  let hoveredChain: string | null = null
  let mouseInCanvas = false
  
  // Rotation state
  let rotationAngle = 0
  let rotationPaused = false
  
  // Selection state for highlighting connections
  let selectedFromChain: string | null = $state(null)
  let selectedToChain: string | null = $state(null)
  
  // Audio context for sound effects
  let audioContext: AudioContext | null = null
  let soundEnabled = true
  
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

  // Track transfers we've already visualized to avoid duplicates
  let seenTransfers = new Set<string>()
  let lastProcessedSortOrder: string | null = null
  
  // Transfer queue with future scheduling for fake "live" feel
  let scheduledTransfers: Array<{
    transfer: any
    scheduledTime: number
  }> = []
  const FUTURE_BUFFER_SECONDS = 10
  
  // Loading state
  let isLoading = $state(true)

  // Animation constants  
  const CHAIN_BASE_SIZE = 12
  const PARTICLE_SPEED = 0.02

  function getChainColor(chainId: string): string {
    return COLOR_CONFIG.chainDefault
  }

  function setupChainNodes() {
    if (!Option.isSome(chains.data)) return
    
    const chainData = chains.data.value
    if (!chainData || chainData.length === 0) return
    
    // Clear existing nodes
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
        color: getChainColor(chain.universal_chain_id),
        activity: 0,
        displayName: chain.display_name || chain.chain_id,
        glowColor: COLOR_CONFIG.chainHit, // White glow when hit
        glowIntensity: 0
      })
    })
    
    console.log(`Set up ${chainNodes.size} chain nodes`)
  }

  function createParticleFromTransfer(transfer: any) {
    if (!chainNodes.has(transfer.source_chain.universal_chain_id) || 
        !chainNodes.has(transfer.destination_chain.universal_chain_id)) {
      return
    }

    const fromNode = chainNodes.get(transfer.source_chain.universal_chain_id)!
    const toNode = chainNodes.get(transfer.destination_chain.universal_chain_id)!
    
    // Increase activity for both chains (more subtle)
    fromNode.activity = Math.min(fromNode.activity + 0.5, 3)
    toNode.activity = Math.min(toNode.activity + 0.5, 3)
    
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
      value: parseFloat(transfer.base_amount) || 1,
      progress: 0,
      color: COLOR_CONFIG.particle,
      size: 3 // Small, consistent dot size for sleek look
    })
  }

  function checkHover() {
    hoveredChain = null
    chainNodes.forEach((node, chainId) => {
      const distance = Math.sqrt(
        (mouseX - node.x) * (mouseX - node.x) + 
        (mouseY - node.y) * (mouseY - node.y)
      )
      if (distance <= node.size + 10) { // 10px hover buffer
        hoveredChain = chainId
      }
    })
    
    // Update cursor style
    if (canvas) {
      canvas.style.cursor = hoveredChain ? 'pointer' : 'default'
    }
  }

  function handleChainClick() {
    console.log('Click detected, hoveredChain:', hoveredChain)
    
    if (!hoveredChain) {
      // Clicked on empty space - clear selection
      selectedFromChain = null
      selectedToChain = null
      console.log('Cleared selection')
      return
    }
    
    if (!selectedFromChain) {
      // First click - select source chain
      selectedFromChain = hoveredChain
      selectedToChain = null
      console.log(`Selected source chain: ${hoveredChain}`)
      console.log('Current state:', { selectedFromChain, selectedToChain })
    } else if (!selectedToChain && hoveredChain !== selectedFromChain) {
      // Second click - select destination chain (different from source)
      selectedToChain = hoveredChain
      console.log(`Selected destination chain: ${hoveredChain}`)
      console.log(`Highlighting connection: ${selectedFromChain} → ${selectedToChain}`)
      console.log('Current state:', { selectedFromChain, selectedToChain })
    } else {
      // Reset selection if clicking same chain or third click
      selectedFromChain = hoveredChain
      selectedToChain = null
      console.log(`Reset selection. New source chain: ${hoveredChain}`)
      console.log('Current state:', { selectedFromChain, selectedToChain })
    }
  }

  // Audio functions for transaction sounds
  function initAudio() {
    try {
      audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    } catch (error) {
      console.warn('Web Audio API not supported:', error)
      soundEnabled = false
    }
  }

  function playTransactionSound(value: number) {
    if (!audioContext || !soundEnabled) return
    
    try {
      // Resume audio context if suspended (required by some browsers)
      if (audioContext.state === 'suspended') {
        audioContext.resume()
      }

      // Create different sounds based on transaction value
      const frequency = Math.min(800 + Math.log(value + 1) * 100, 1500) // Higher pitch for larger values
      const duration = 0.15 + Math.min(Math.log(value + 1) * 0.02, 0.1) // Slightly longer for larger values
      
      // Create oscillator for the main tone (like a coin drop)
      const oscillator = audioContext.createOscillator()
      const gainNode = audioContext.createGain()
      
      oscillator.connect(gainNode)
      gainNode.connect(audioContext.destination)
      
      // Bell-like sound with quick decay
      oscillator.type = 'sine'
      oscillator.frequency.setValueAtTime(frequency, audioContext.currentTime)
      oscillator.frequency.exponentialRampToValueAtTime(frequency * 0.5, audioContext.currentTime + duration)
      
      // Volume envelope (quick attack, exponential decay)
      gainNode.gain.setValueAtTime(0, audioContext.currentTime)
      gainNode.gain.linearRampToValueAtTime(0.1, audioContext.currentTime + 0.01) // Quick attack
      gainNode.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + duration)
      
      oscillator.start(audioContext.currentTime)
      oscillator.stop(audioContext.currentTime + duration)
      
      // Add a subtle metallic overtone for richness
      const overtone = audioContext.createOscillator()
      const overtoneGain = audioContext.createGain()
      
      overtone.connect(overtoneGain)
      overtoneGain.connect(audioContext.destination)
      
      overtone.type = 'triangle'
      overtone.frequency.setValueAtTime(frequency * 2.5, audioContext.currentTime)
      overtone.frequency.exponentialRampToValueAtTime(frequency * 1.2, audioContext.currentTime + duration * 0.7)
      
      overtoneGain.gain.setValueAtTime(0, audioContext.currentTime)
      overtoneGain.gain.linearRampToValueAtTime(0.03, audioContext.currentTime + 0.005)
      overtoneGain.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + duration * 0.7)
      
      overtone.start(audioContext.currentTime)
      overtone.stop(audioContext.currentTime + duration * 0.7)
      
    } catch (error) {
      console.warn('Failed to play transaction sound:', error)
    }
  }

  function animate() {
    if (!ctx) return

    // Check for hover
    checkHover()

    // Clear canvas with transparent background
    ctx.clearRect(0, 0, canvasWidth, canvasHeight)

    // Update particles (but don't draw them yet)
    particles = particles.filter(particle => {
      particle.progress += PARTICLE_SPEED
      
      if (particle.progress >= 1) {
        // Particle reached destination
        const toNode = chainNodes.get(particle.toChain)
        if (toNode) {
          toNode.pulseSize = toNode.size * 1.5
          // Set white glow when hit by transaction
          toNode.glowColor = COLOR_CONFIG.chainHit
          toNode.glowIntensity = 1.0
          // Play transaction completion sound
          playTransactionSound(particle.value)
        }
        return false
      }

      // Smoother interpolation with ease-in-out (direct position calculation)
      const t = particle.progress
      const easedT = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2 // Ease in-out cubic
      particle.x = particle.startX + (particle.targetX - particle.startX) * easedT
      particle.y = particle.startY + (particle.targetY - particle.startY) * easedT

      return true
    })

    // Update and draw chain nodes
    if (chainNodes.size === 0) {
      // Just continue animating without text - cleaner look
      animationFrame = requestAnimationFrame(animate)
      return
    }
    
    // First, draw all normal connection lines
    const nodeArray = Array.from(chainNodes.entries())
    let selectedConnectionPairs: Array<[any, any]> = []
    
    for (let i = 0; i < nodeArray.length; i++) {
      for (let j = i + 1; j < nodeArray.length; j++) {
        const [chainId1, node1] = nodeArray[i]
        const [chainId2, node2] = nodeArray[j]
        
        // Check if this is the selected connection
        const isSelectedConnection = (
          (selectedFromChain === chainId1 && selectedToChain === chainId2) ||
          (selectedFromChain === chainId2 && selectedToChain === chainId1)
        )
        
        if (isSelectedConnection) {
          // Save selected connections for later drawing
          selectedConnectionPairs.push([node1, node2])
        } else {
          // Draw normal connection lines
          ctx.beginPath()
          ctx.moveTo(node1.x, node1.y)
          ctx.lineTo(node2.x, node2.y)
          ctx.strokeStyle = COLOR_CONFIG.connectionDefault
          ctx.lineWidth = 1
          ctx.globalAlpha = 1
          ctx.stroke()
        }
      }
    }
    
    ctx.globalAlpha = 1 // Reset alpha
    
    // Then, update and draw all nodes
    chainNodes.forEach((node, chainId) => {
      // Decay activity (faster decay for more subtle effect)
      node.activity = Math.max(node.activity - 0.08, 0)
      
      // Decay glow intensity
      node.glowIntensity = Math.max(node.glowIntensity - 0.05, 0)
      
      // Update pulse
      node.pulseSize = node.pulseSize + (node.size - node.pulseSize) * 0.1
      
      // Draw subtle pulse effect
      if (node.activity > 0) {
        const pulseRadius = node.size + node.activity * 2.5
        const pulseGradient = ctx.createRadialGradient(
          node.x, node.y, node.size,
          node.x, node.y, pulseRadius
        )
        pulseGradient.addColorStop(0, node.color + '20') // 20% opacity
        pulseGradient.addColorStop(1, node.color + '00') // Transparent
        
        ctx.beginPath()
        ctx.arc(node.x, node.y, pulseRadius, 0, 2 * Math.PI)
        ctx.fillStyle = pulseGradient
        ctx.fill()
      }
      
      // Draw node with sophisticated styling (more subtle size increase)
      const nodeRadius = node.size + node.activity * 0.8
      
      // Check if this node is selected
      const isSelected = (chainId === selectedFromChain || chainId === selectedToChain)
      
      // Outer glow (subtle for selected nodes)
      const outerGlow = ctx.createRadialGradient(
        node.x, node.y, 0,
        node.x, node.y, nodeRadius * (isSelected ? 1.8 : 1.5)
      )
      const glowOpacity = isSelected ? '60' : '40'
      outerGlow.addColorStop(0, (isSelected ? COLOR_CONFIG.chainSelected : node.color) + glowOpacity)
      outerGlow.addColorStop(1, (isSelected ? COLOR_CONFIG.chainSelected : node.color) + '00')
      
      ctx.beginPath()
      ctx.arc(node.x, node.y, nodeRadius * (isSelected ? 1.8 : 1.5), 0, 2 * Math.PI)
      ctx.fillStyle = outerGlow
      ctx.fill()
      
      // Main node with glow color blending
      ctx.beginPath()
      ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)
      
      // Blend original color with glow color based on glow intensity
      if (node.glowIntensity > 0) {
        // Create a blended color
        const glowAmount = node.glowIntensity
        const baseAmount = 1 - glowAmount
        // Simple color blending (assumes hex colors)
        const r1 = parseInt(node.color.slice(1, 3), 16)
        const g1 = parseInt(node.color.slice(3, 5), 16)
        const b1 = parseInt(node.color.slice(5, 7), 16)
        const r2 = parseInt(node.glowColor.slice(1, 3), 16)
        const g2 = parseInt(node.glowColor.slice(3, 5), 16)
        const b2 = parseInt(node.glowColor.slice(5, 7), 16)
        
        const r = Math.round(r1 * baseAmount + r2 * glowAmount)
        const g = Math.round(g1 * baseAmount + g2 * glowAmount)
        const b = Math.round(b1 * baseAmount + b2 * glowAmount)
        
        ctx.fillStyle = `rgb(${r}, ${g}, ${b})`
      } else {
        ctx.fillStyle = node.color
      }
      ctx.fill()
      
      // Subtle border
      ctx.beginPath()
      ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI)
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)'
      ctx.lineWidth = 1
      ctx.stroke()
      
      // Draw chain name only on hover
      if (hoveredChain === chainId) {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.9)'
        ctx.font = '10px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'  // Reduced from 11px
        ctx.textAlign = 'center'
        
        // Add subtle text shadow for better readability
        ctx.shadowColor = 'rgba(0, 0, 0, 0.7)'
        ctx.shadowBlur = 4
        ctx.shadowOffsetX = 0
        ctx.shadowOffsetY = 1
        
        ctx.fillText(node.displayName, node.x, node.y - nodeRadius - 8)
        
        // Reset shadow
        ctx.shadowColor = 'transparent'
        ctx.shadowBlur = 0
        ctx.shadowOffsetX = 0
        ctx.shadowOffsetY = 0
      }
    })

    // Draw selected connections on top
    selectedConnectionPairs.forEach(([node1, node2]) => {
      ctx.beginPath()
      ctx.moveTo(node1.x, node1.y)
      ctx.lineTo(node2.x, node2.y)
      ctx.strokeStyle = COLOR_CONFIG.connectionSelected
      ctx.lineWidth = 1
      ctx.globalAlpha = 0.6
      ctx.stroke()
    })
    ctx.globalAlpha = 1 // Reset alpha

    // Finally, draw particles on top of everything
    particles.forEach(particle => {
      // Draw glowing particle (keeping your size of 3)
      // Outer glow
      const glowGradient = ctx.createRadialGradient(
        particle.x, particle.y, 0,
        particle.x, particle.y, particle.size * 2.5
      )
      glowGradient.addColorStop(0, COLOR_CONFIG.particleGlow) // 40% opacity
      glowGradient.addColorStop(1, particle.color + '00') // Transparent
      
      ctx.beginPath()
      ctx.arc(particle.x, particle.y, particle.size * 2.5, 0, 2 * Math.PI)
      ctx.fillStyle = glowGradient
      ctx.fill()
      
      // Main particle (your original size 3)
      ctx.beginPath()
      ctx.arc(particle.x, particle.y, particle.size, 0, 2 * Math.PI)
      ctx.fillStyle = particle.color
      ctx.globalAlpha = 1
      ctx.fill()
      
      ctx.globalAlpha = 1
    })

    animationFrame = requestAnimationFrame(animate)
  }

  onMount(() => {
    // Initialize canvas with high-DPI support for sharp rendering
    if (canvas) {
      ctx = canvas.getContext('2d')!
      
      // Get device pixel ratio for sharp rendering on high-DPI displays
      const dpr = window.devicePixelRatio || 1
      
      // Set actual canvas size
      canvas.width = canvasWidth * dpr
      canvas.height = canvasHeight * dpr
      
      // Scale the canvas back down using CSS
      canvas.style.width = canvasWidth + 'px'
      canvas.style.height = canvasHeight + 'px'
      
      // Scale the drawing context so everything draws at the correct size
      ctx.scale(dpr, dpr)
      
      // Enable crisp rendering
      ctx.imageSmoothingEnabled = false
      
      // Add mouse event listeners for hover detection and clicking
      canvas.addEventListener('mousemove', (e) => {
        const rect = canvas.getBoundingClientRect()
        mouseX = e.clientX - rect.left
        mouseY = e.clientY - rect.top
      })
      
      canvas.addEventListener('mouseleave', () => {
        hoveredChain = null
        canvas.style.cursor = 'default'
      })
      
      canvas.addEventListener('click', handleChainClick)
      
      // Initialize audio on first user interaction (required by browsers)
      canvas.addEventListener('click', () => {
        if (!audioContext) {
          initAudio()
        }
      }, { once: true })
      
      animate()
    }

    // Initialize transfer queries to start fetching data
    const initQueries = async () => {
      try {
        // Start transfer queries with built-in refetching (query handles automatic updates)
        await transferList.runEffect(transferListLatestQuery(50))
      } catch (error) {
        console.error('Failed to initialize queries:', error)
      }
    }

    initQueries()

    // Start time-based streaming - check for scheduled transfers
    const startStreaming = () => {
      const checkScheduled = () => {
        const now = Date.now()
        
        // Update loading state if transfers are about to appear (within 2 seconds)
        if (isLoading && scheduledTransfers.length > 0) {
          const soonestTransfer = Math.min(...scheduledTransfers.map(t => t.scheduledTime))
          if (soonestTransfer - now <= 2000) {
            isLoading = false
          }
        }
        
        // Execute ready transfers
        const readyTransfers = scheduledTransfers.filter(item => item.scheduledTime <= now)
        if (readyTransfers.length > 0) {
          readyTransfers.forEach(item => createParticleFromTransfer(item.transfer))
          scheduledTransfers = scheduledTransfers.filter(item => item.scheduledTime > now)
          console.log(`Executed ${readyTransfers.length} scheduled transfers (${scheduledTransfers.length} remaining)`)
        }
        
        setTimeout(checkScheduled, 50)
      }
      
      checkScheduled()
    }

    startStreaming()

    return () => {
      if (animationFrame) {
        cancelAnimationFrame(animationFrame)
      }
      // Clean up transfer query fiber
      transferList.interruptFiber()
    }
  })

  // Setup chain nodes when chains data becomes available
  $effect(() => {
    if (Option.isSome(chains.data)) {
      setupChainNodes()
    }
  })

  // React to new transfers - create smooth streaming animation
  $effect(() => {
    if (Option.isSome(transferList.data)) {
      const transfers = transferList.data.value
      
      // Find truly NEW transfers by comparing sort_order
      const newTransfers = transfers.filter(transfer => {
        // Skip if we've already seen this exact transfer
        if (seenTransfers.has(transfer.packet_hash)) {
          return false
        }
        
        // If this is our first fetch, process recent transfers
        if (lastProcessedSortOrder === null) {
          return true
        }
        
        // Only include transfers newer than our last processed sort_order
        return transfer.sort_order > lastProcessedSortOrder
      })
      
      if (newTransfers.length > 0) {
        console.log(`Found ${newTransfers.length} new transfers to animate`)
        
        // Update our tracking (sort_order is a string, so we take the lexically largest)
        const latestSortOrder = newTransfers
          .map(t => t.sort_order)
          .sort()
          .pop()
        lastProcessedSortOrder = latestSortOrder || lastProcessedSortOrder
        
        // Schedule new transfers in the future for fake "live" feel
        const now = Date.now()
        const futureBaseTime = now + (FUTURE_BUFFER_SECONDS * 1000)
        
        newTransfers.forEach((transfer, index) => {
          seenTransfers.add(transfer.packet_hash)
          
          // Spread transfers over the next 10 seconds for natural flow
          const spreadTime = (index / Math.max(newTransfers.length - 1, 1)) * 10000 // 10 seconds
          const jitter = (Math.random() - 0.5) * 2000 // ±1 second jitter
          const scheduledTime = futureBaseTime + spreadTime + jitter
          
          scheduledTransfers.push({
            transfer,
            scheduledTime
          })
        })
        
        // Sort by scheduled time to ensure proper order
        scheduledTransfers.sort((a, b) => a.scheduledTime - b.scheduledTime)
        
        // Loading state will be updated in the streaming loop
        
        console.log(`Scheduled ${newTransfers.length} transfers for future playback (${scheduledTransfers.length} total in pipeline)`)
      }
      
      // Clean up old seen transfers to prevent memory bloat (keep last 1000)
      if (seenTransfers.size > 1000) {
        const oldTransfers = Array.from(seenTransfers).slice(0, 500)
        oldTransfers.forEach(hash => seenTransfers.delete(hash))
      }
    }
  })
</script>

<Sections>
    <Card class="flex items-center justify-center relative">   
        {#if selectedFromChain && selectedToChain}
        <div class="absolute top-4 left-4 z-10 p-3 rounded-lg text-white min-w-48" style="background-color: {COLOR_CONFIG.uiBackground};">
          <div class="text-xs mb-1" style="color: {COLOR_CONFIG.uiTextSecondary};">Route Latency</div>
          <div class="text-sm font-medium mb-2" style="color: {COLOR_CONFIG.uiText};">
            {chainNodes.get(selectedFromChain)?.displayName || selectedFromChain} → 
            {chainNodes.get(selectedToChain)?.displayName || selectedToChain}
          </div>
          
          <div class="text-xs mt-2" style="color: {COLOR_CONFIG.uiTextMuted};">
            Click elsewhere to clear selection
          </div>
        </div>
      {/if}     
        <canvas 
          bind:this={canvas}
          class:opacity-50={isLoading}
          style="background: transparent;"
        ></canvas>
        
        {#if isLoading}
          <div class="absolute inset-0 flex items-center justify-center bg-black/20 rounded-lg">
            <div class="flex flex-col items-center gap-3">
              <div class="animate-spin h-8 w-8 border-2 border-t-transparent rounded-full" style="border-color: {COLOR_CONFIG.loadingSpinner}; border-top-color: transparent;"></div>
            </div>
          </div>
        {/if}
    </Card>
</Sections> 