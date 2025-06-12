class TransactionAudio {
  private audioContext: AudioContext | null = null
  private soundEnabled = false // Start muted by default
  private audioCache: Map<string, AudioBuffer> = new Map()
  private offlineContext: OfflineAudioContext | null = null

  constructor() {
    this.init()
  }

  private init() {
    try {
      // @ts-expect-error - webkitAudioContext is a legacy webkit property
      this.audioContext = new (window.AudioContext || window.webkitAudioContext)()
      // Create offline context for pre-rendering audio
      this.offlineContext = new OfflineAudioContext(2, 44100 * 0.5, 44100) // 500ms stereo buffer
    } catch (error) {
      console.warn("Web Audio API not supported:", error)
      this.soundEnabled = false
    }
  }

  // Generate cache key for a specific sound
  private getCacheKey(frequency: number, waveType: OscillatorType, duration: number): string {
    return `${frequency.toFixed(1)}_${waveType}_${Math.round(duration * 1000)}`
  }

  // Pre-render and cache a single sound
  private async createCachedSound(frequency: number, waveType: OscillatorType, duration: number): Promise<AudioBuffer | null> {
    if (!this.offlineContext) return null

    const cacheKey = this.getCacheKey(frequency, waveType, duration)
    
    // Return cached version if exists
    if (this.audioCache.has(cacheKey)) {
      return this.audioCache.get(cacheKey)!
    }

    try {
      // Create a new offline context for this sound
      const offlineCtx = new OfflineAudioContext(2, Math.ceil(44100 * duration * 1.5), 44100)
      
      // Create the sound using same logic as live version
      const oscillator = offlineCtx.createOscillator()
      const gainNode = offlineCtx.createGain()
      const filterNode = offlineCtx.createBiquadFilter()
      const leftGain = offlineCtx.createGain()
      const rightGain = offlineCtx.createGain()
      const merger = offlineCtx.createChannelMerger(2)

      // Connect for stereo output
      oscillator.connect(filterNode)
      filterNode.connect(gainNode)
      gainNode.connect(leftGain)
      gainNode.connect(rightGain)
      leftGain.connect(merger, 0, 0)
      rightGain.connect(merger, 0, 1)
      merger.connect(offlineCtx.destination)

      // Set up the sound parameters
      oscillator.type = waveType
      oscillator.frequency.setValueAtTime(frequency, 0)

      filterNode.type = "lowpass"
      filterNode.frequency.setValueAtTime(frequency * 4, 0)
      filterNode.Q.setValueAtTime(0.8, 0)

      // Envelope
      gainNode.gain.setValueAtTime(0, 0)
      gainNode.gain.linearRampToValueAtTime(0.18, 0.005)
      gainNode.gain.exponentialRampToValueAtTime(0.001, duration)

      // Create harmonic
      const harmonic = offlineCtx.createOscillator()
      const harmonicGain = offlineCtx.createGain()
      const harmonicFilter = offlineCtx.createBiquadFilter()

      harmonic.connect(harmonicFilter)
      harmonicFilter.connect(harmonicGain)
      harmonicGain.connect(leftGain)
      harmonicGain.connect(rightGain)

      const harmonicMultiplier = waveType === "sine" ? 2 : 
                                waveType === "triangle" ? 3 : 
                                waveType === "square" ? 1.5 : 2.5

      harmonic.type = "sine"
      harmonic.frequency.setValueAtTime(frequency * harmonicMultiplier, 0)

      harmonicFilter.type = "lowpass"
      harmonicFilter.frequency.setValueAtTime(frequency * 3, 0)
      harmonicFilter.Q.setValueAtTime(1, 0)

      harmonicGain.gain.setValueAtTime(0, 0)
      harmonicGain.gain.linearRampToValueAtTime(0.06, 0.001)
      harmonicGain.gain.exponentialRampToValueAtTime(0.001, duration * 0.7)

      // Start oscillators
      oscillator.start(0)
      oscillator.stop(duration)
      harmonic.start(0)
      harmonic.stop(duration * 0.7)

      // Render and cache
      const buffer = await offlineCtx.startRendering()
      this.audioCache.set(cacheKey, buffer)
      return buffer

    } catch (error) {
      console.warn("Failed to create cached sound:", error)
      return null
    }
  }

     // Pre-cache common sounds based on chain values
   async preloadChainSounds() {
     if (!this.audioContext) return

     const commonDurations = [0.08, 0.1, 0.12] // Common durations we use
     
     // Pre-cache sounds for common chain value ranges
     const promises: Promise<AudioBuffer | null>[] = []
     
     for (let chainValue = 100; chainValue <= 900; chainValue += 100) {
       const frequency = this.getChainFrequency(chainValue)
       const waveType = this.getChainWaveType(chainValue)
       
       for (const duration of commonDurations) {
         promises.push(this.createCachedSound(frequency, waveType, duration))
       }
     }

     // Wait for all sounds to be cached
     await Promise.all(promises)
     console.log(`Preloaded ${this.audioCache.size} cached sounds`)
   }

  async resumeIfNeeded() {
    if (!this.audioContext || !this.soundEnabled) {
      return
    }

    try {
      // Resume audio context if suspended (required by some browsers)
      if (this.audioContext.state === "suspended") {
        await this.audioContext.resume()
      }
    } catch (error) {
      console.warn("Failed to resume audio context:", error)
    }
  }

     // Generate a consistent numeric value from chain ID
   private getChainValue(chainId: string): number {
     if (!chainId) return 1
     
     // Simple hash function to convert string to number
     let hash = 0
     for (let i = 0; i < chainId.length; i++) {
       const char = chainId.charCodeAt(i)
       hash = ((hash << 5) - hash) + char
       hash = hash & hash // Convert to 32-bit integer
     }
     
     // Convert to positive number and scale to reasonable range (1-1000)
     return Math.abs(hash % 1000) + 1
   }

   // Map chain value to frequency (musical notes)
   private getChainFrequency(chainValue: number): number {
     // Map to musical scale - pentatonic for nice harmonies
     const notes = [
       261.63, // C4
       293.66, // D4
       329.63, // E4
       392.00, // G4
       440.00, // A4
       523.25, // C5
       587.33, // D5
       659.25, // E5
       783.99, // G5
       880.00  // A5
     ]
     
     const noteIndex = chainValue % notes.length
     return notes[noteIndex]
   }

   // Map chain value to wave type for timbral variety
   private getChainWaveType(chainValue: number): OscillatorType {
     const waveTypes: OscillatorType[] = ["sine", "square", "sawtooth", "triangle"]
     const typeIndex = Math.floor(chainValue / 250) % waveTypes.length
     return waveTypes[typeIndex]
   }

  playSound(transactionValue: number, sourceChainId?: string, destChainId?: string) {
    if (!this.audioContext || !this.soundEnabled) {
      return
    }

    try {
      this.resumeIfNeeded()

      // Combine transaction value with chain-based values
      const sourceValue = sourceChainId ? this.getChainValue(sourceChainId) : 0
      const destValue = destChainId ? this.getChainValue(destChainId) : 0
      const combinedValue = transactionValue + sourceValue + destValue

      // BTC ticker style sounds based on combined value
      const logValue = Math.log(combinedValue + 1)
      
      if (logValue > 15) {
        // Large transaction - rapid ticker sequence
        this.playLargeTickerSound(combinedValue, sourceChainId, destChainId)
      } else if (logValue > 10) {
        // Medium transaction - double beep
        this.playMediumTickerSound(combinedValue, sourceChainId, destChainId)
      } else {
        // Regular transaction - single ticker beep
        this.playTickerBeep(combinedValue, sourceChainId, destChainId)
      }

    } catch (error) {
      console.warn("Failed to play sound:", error)
    }
  }

        private playLargeTickerSound(_value: number, sourceChainId?: string, destChainId?: string) {
     if (!this.audioContext) return

     // Rapid ticker sequence using chain tones
     const sourceValue = sourceChainId ? this.getChainValue(sourceChainId) : 500
     const destValue = destChainId ? this.getChainValue(destChainId) : 600
     const sourceFreq = this.getChainFrequency(sourceValue)
     const destFreq = this.getChainFrequency(destValue)
     const sourceWave = this.getChainWaveType(sourceValue)
     const destWave = this.getChainWaveType(destValue)
     
     const beepCount = 5
     for (let i = 0; i < beepCount; i++) {
       setTimeout(() => {
         const freq = i % 2 === 0 ? sourceFreq : destFreq // Alternate between source and dest
         const wave = i % 2 === 0 ? sourceWave : destWave
         const pan = (Math.random() - 0.5) * 0.6 // Slight random stereo
         this.createChainTickerBeep(freq + (i * 50), 0.08, pan, wave) // Rising pitch sequence
       }, i * 60) // Very rapid succession
     }
   }

   private playMediumTickerSound(_value: number, sourceChainId?: string, destChainId?: string) {
     if (!this.audioContext) return

     // Double beep using actual chain tones
     const sourceValue = sourceChainId ? this.getChainValue(sourceChainId) : 500
     const destValue = destChainId ? this.getChainValue(destChainId) : 600
     const sourceFreq = this.getChainFrequency(sourceValue)
     const destFreq = this.getChainFrequency(destValue)
     const sourceWave = this.getChainWaveType(sourceValue)
     const destWave = this.getChainWaveType(destValue)

     // Source chain sound (left)
     this.createChainTickerBeep(sourceFreq, 0.1, -0.3, sourceWave)
     
     setTimeout(() => {
       // Destination chain sound (right)
       this.createChainTickerBeep(destFreq, 0.1, 0.3, destWave)
     }, 80)
   }

   private playTickerBeep(value: number, sourceChainId?: string, destChainId?: string) {
     if (!this.audioContext) return

     // Use chain values to determine tone characteristics
     const sourceValue = sourceChainId ? this.getChainValue(sourceChainId) : 500
     const destValue = destChainId ? this.getChainValue(destChainId) : 500

     // Create source tone (left channel)
     const sourceFreq = this.getChainFrequency(sourceValue)
     const sourceWave = this.getChainWaveType(sourceValue)
     this.createChainTickerBeep(sourceFreq, 0.12, -0.3, sourceWave)

     // Create destination tone (right channel) after small delay
     setTimeout(() => {
       const destFreq = this.getChainFrequency(destValue)
       const destWave = this.getChainWaveType(destValue)
       this.createChainTickerBeep(destFreq, 0.1, 0.3, destWave)
     }, 80)
   }

     private async playCachedSound(frequency: number, duration: number, pan: number, waveType: OscillatorType) {
     if (!this.audioContext) return

     const cacheKey = this.getCacheKey(frequency, waveType, duration)
     let buffer = this.audioCache.get(cacheKey)

     // If not cached, create it on-demand
     if (!buffer) {
       const newBuffer = await this.createCachedSound(frequency, waveType, duration)
       if (!newBuffer) return // Failed to create
       buffer = newBuffer
     }

     // Play the cached buffer
     const source = this.audioContext.createBufferSource()
     const gainNode = this.audioContext.createGain()
     const panNode = this.audioContext.createStereoPanner()

     source.buffer = buffer
     source.connect(gainNode)
     gainNode.connect(panNode)
     panNode.connect(this.audioContext.destination)

     // Apply panning
     panNode.pan.setValueAtTime(pan, this.audioContext.currentTime)
     
     // Optional gain adjustment
     gainNode.gain.setValueAtTime(1, this.audioContext.currentTime)

     source.start(this.audioContext.currentTime)
   }

   private createChainTickerBeep(frequency: number, duration: number, pan: number, waveType: OscillatorType) {
     // Use cached version for better performance
     this.playCachedSound(frequency, duration, pan, waveType)
   }



  playTransactionSound(value: number) {
    if (!this.audioContext || !this.soundEnabled) {
      return
    }

    try {
      this.resumeIfNeeded()

      // Create different sounds based on transaction value
      const frequency = Math.min(800 + Math.log(value + 1) * 100, 1500) // Higher pitch for larger values
      const duration = 0.15 + Math.min(Math.log(value + 1) * 0.02, 0.1) // Slightly longer for larger values

      // Create oscillator for the main tone (like a coin drop)
      const oscillator = this.audioContext.createOscillator()
      const gainNode = this.audioContext.createGain()

      oscillator.connect(gainNode)
      gainNode.connect(this.audioContext.destination)

      // Bell-like sound with quick decay
      oscillator.type = "sine"
      oscillator.frequency.setValueAtTime(frequency, this.audioContext.currentTime)
      oscillator.frequency.exponentialRampToValueAtTime(
        frequency * 0.5,
        this.audioContext.currentTime + duration,
      )

      // Volume envelope (quick attack, exponential decay)
      gainNode.gain.setValueAtTime(0, this.audioContext.currentTime)
      gainNode.gain.linearRampToValueAtTime(0.1, this.audioContext.currentTime + 0.01) // Quick attack
      gainNode.gain.exponentialRampToValueAtTime(0.001, this.audioContext.currentTime + duration)

      oscillator.start(this.audioContext.currentTime)
      oscillator.stop(this.audioContext.currentTime + duration)

      // Add a subtle metallic overtone for richness
      const overtone = this.audioContext.createOscillator()
      const overtoneGain = this.audioContext.createGain()

      overtone.connect(overtoneGain)
      overtoneGain.connect(this.audioContext.destination)

      overtone.type = "triangle"
      overtone.frequency.setValueAtTime(frequency * 2.5, this.audioContext.currentTime)
      overtone.frequency.exponentialRampToValueAtTime(
        frequency * 1.2,
        this.audioContext.currentTime + duration * 0.7,
      )

      overtoneGain.gain.setValueAtTime(0, this.audioContext.currentTime)
      overtoneGain.gain.linearRampToValueAtTime(0.03, this.audioContext.currentTime + 0.005)
      overtoneGain.gain.exponentialRampToValueAtTime(
        0.001,
        this.audioContext.currentTime + duration * 0.7,
      )

      overtone.start(this.audioContext.currentTime)
      overtone.stop(this.audioContext.currentTime + duration * 0.7)
    } catch (error) {
      console.warn("Failed to play transaction sound:", error)
    }
  }

  enable() {
    this.soundEnabled = true
    // Preload sounds when audio is first enabled
    this.preloadChainSounds()
  }

  disable() {
    this.soundEnabled = false
  }

  isEnabled() {
    return this.soundEnabled
  }
}

// Export a singleton instance
export const transactionAudio = new TransactionAudio()
