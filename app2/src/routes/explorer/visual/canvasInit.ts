/**
 * Robust canvas initialization utility that handles timing issues
 * Ensures canvas has proper dimensions before calling initialization callbacks
 */

export interface CanvasInitOptions {
  canvas: HTMLCanvasElement
  container: HTMLElement
  onInitialized: () => void
  maxRetries?: number
  retryDelay?: number
}

export function initializeCanvasRobustly(options: CanvasInitOptions): void {
  const { container, onInitialized, maxRetries = 20, retryDelay = 50 } = options
  let attempts = 0

  const tryInitialize = () => {
    attempts++
    
    // Get container dimensions
    const rect = container.getBoundingClientRect()
    const width = rect.width
    const height = rect.height
    
    // Check if we have valid dimensions
    if (width > 0 && height > 0) {
      // Success - container has dimensions
      onInitialized()
      return
    }
    
    // If we've exceeded max retries, force initialization anyway
    if (attempts >= maxRetries) {
      console.warn('Canvas initialization: Max retries exceeded, forcing initialization')
      onInitialized()
      return
    }
    
    // Retry after delay
    setTimeout(tryInitialize, retryDelay)
  }

  // Start the initialization process
  // Use both requestAnimationFrame and immediate retry for maximum compatibility
  requestAnimationFrame(() => {
    tryInitialize()
  })
}

/**
 * Enhanced version that also sets up resize observer and event cleanup
 */
export interface EnhancedCanvasInitOptions extends CanvasInitOptions {
  onResize?: () => void
  eventListeners?: Array<{
    element: HTMLElement | Window
    event: string
    handler: (event: Event) => void
    options?: AddEventListenerOptions
  }>
}

export function initializeCanvasWithCleanup(options: EnhancedCanvasInitOptions): () => void {
  const { container, onResize, eventListeners = [] } = options
  
  let resizeObserver: ResizeObserver | null = null
  
  // Setup resize observer if resize callback provided
  if (onResize) {
    resizeObserver = new ResizeObserver(onResize)
    resizeObserver.observe(container)
  }
  
  // Add event listeners
  eventListeners.forEach(({ element, event, handler, options: listenerOptions }) => {
    element.addEventListener(event, handler, listenerOptions)
  })
  
  // Initialize canvas
  initializeCanvasRobustly(options)
  
  // Return cleanup function
  return () => {
    resizeObserver?.disconnect()
    eventListeners.forEach(({ element, event, handler, options: listenerOptions }) => {
      element.removeEventListener(event, handler, listenerOptions)
    })
  }
} 