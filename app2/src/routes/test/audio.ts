class TransactionAudio {
  private audioContext: AudioContext | null = null
  private soundEnabled = false // Start muted by default

  constructor() {
    this.init()
  }

  private init() {
    try {
      // @ts-expect-error - webkitAudioContext is a legacy webkit property
      this.audioContext = new (window.AudioContext || window.webkitAudioContext)()
    } catch (error) {
      console.warn('Web Audio API not supported:', error)
      this.soundEnabled = false
    }
  }

  async resumeIfNeeded() {
    if (!this.audioContext || !this.soundEnabled) return
    
    try {
      // Resume audio context if suspended (required by some browsers)
      if (this.audioContext.state === 'suspended') {
        await this.audioContext.resume()
      }
    } catch (error) {
      console.warn('Failed to resume audio context:', error)
    }
  }

  playTransactionSound(value: number) {
    if (!this.audioContext || !this.soundEnabled) return
    
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
      oscillator.type = 'sine'
      oscillator.frequency.setValueAtTime(frequency, this.audioContext.currentTime)
      oscillator.frequency.exponentialRampToValueAtTime(frequency * 0.5, this.audioContext.currentTime + duration)
      
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
      
      overtone.type = 'triangle'
      overtone.frequency.setValueAtTime(frequency * 2.5, this.audioContext.currentTime)
      overtone.frequency.exponentialRampToValueAtTime(frequency * 1.2, this.audioContext.currentTime + duration * 0.7)
      
      overtoneGain.gain.setValueAtTime(0, this.audioContext.currentTime)
      overtoneGain.gain.linearRampToValueAtTime(0.03, this.audioContext.currentTime + 0.005)
      overtoneGain.gain.exponentialRampToValueAtTime(0.001, this.audioContext.currentTime + duration * 0.7)
      
      overtone.start(this.audioContext.currentTime)
      overtone.stop(this.audioContext.currentTime + duration * 0.7)
      
    } catch (error) {
      console.warn('Failed to play transaction sound:', error)
    }
  }

  enable() {
    this.soundEnabled = true
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