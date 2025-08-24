import { debounce } from "$lib/utils"
import type { Token } from "@unionlabs/sdk"

/**
 * Type definition for form fields managed by this store
 */
export type FormFields = {
  source: string // Source identifier
  destination: string // Destination identifier
  asset: string // Asset type or identifier
  receiver: string // Receiver of the transaction/action
  sender: string
  amount: string // Amount value (stored as string for form handling)
  quoteToken: string // Selected quote token denom on destination chain
}

/**
 * RawIntentsStoreSvelte - A state management class for form fields with URL synchronization
 *
 * Features:
 * - Reactive state for form fields using Svelte 5's $state
 * - Automatic URL parameter synchronization
 * - Debounced URL updates to prevent excessive history entries
 * - Dependency management between fields (resetting related fields)
 * - URL parameter parsing on initialization
 */
export class RawTransferDataSvelte {
  // Reactive state fields using Svelte 5 $state syntax
  source: string = $state("")
  destination: string = $state("")
  asset: string = $state("")
  receiver: string = $state("")
  sender: string = $state("")
  amount: string = $state("")
  quoteToken: string = $state("")

  constructor() {
    // Initialize values from URL parameters if they exist
    this.initFromUrlParams()
  }

  /**
   * Initialize form state from URL parameters
   * This allows for bookmarking or sharing URLs with pre-filled form data
   */
  initFromUrlParams = () => {
    const url = new URL(window.location.href)
    const searchParams = url.searchParams

    // Check if we have any parameters to initialize from
    if ([...searchParams.entries()].length > 0) {
      // Load values from URL parameters if they exist
      const initialValues: Partial<FormFields> = {} // Get each field from URL parameters
      ;(Object.keys(this.cleanState({})) as Array<keyof FormFields>).forEach(field => {
        const paramValue = searchParams.get(field)
        if (paramValue) {
          initialValues[field] = paramValue
        }
      })

      // Set the values without updating URL (to avoid circular updates)
      this.setWithoutUrlUpdate(initialValues)
    } else {
      // If no parameters, clear URL (in case of fragment identifiers or other cruft)
      this.clearUrlParameters()
    }
  }

  /**
   * Clear all URL search parameters
   * Preserves the rest of the URL (path, hash, etc.)
   */
  clearUrlParameters = () => {
    const url = new URL(window.location.href)
    url.search = ""
    history.replaceState({}, "", url.toString())
  }

  /**
   * Helper method to clean and normalize state
   * Ensures all expected fields exist with default empty string values
   * @param state Partial state object to normalize
   * @returns Complete state object with all fields
   */
  cleanState = (state: Partial<FormFields>): FormFields => {
    const {
      source = "",
      destination = "",
      asset = "",
      receiver = "",
      sender = "",
      amount = "",
      quoteToken = "",
    } = state
    return { source, destination, asset, receiver, sender, amount, quoteToken }
  }

  /**
   * Sets field values without updating the URL
   * Used internally to prevent circular updates
   * @param value Partial state object with fields to update
   */
  setWithoutUrlUpdate = (value: Partial<FormFields>) => {
    // Create a new cleaned state with updated values
    const newParams = this.cleanState({
      source: this.source,
      destination: this.destination,
      asset: this.asset,
      receiver: this.receiver,
      sender: this.sender,
      amount: this.amount,
      quoteToken: this.quoteToken,
      ...value,
    })

    // Update state properties
    this.source = newParams.source
    this.destination = newParams.destination
    this.asset = newParams.asset
    this.receiver = newParams.receiver
    this.sender = newParams.sender
    this.amount = newParams.amount
    this.quoteToken = newParams.quoteToken
  }

  /**
   * Debounced URL update function
   * Prevents excessive history entries when values change rapidly
   * Only includes non-empty values in URL parameters
   */
  debouncedUpdateUrl = debounce((params: FormFields) => {
    const url = new URL(window.location.href)

    // Clear existing parameters
    url.search = ""

    // Set new parameters (only if they have a value)
    Object.entries(params).forEach(([key, val]) => {
      if (val) {
        url.searchParams.set(key, val)
      }
    })

    history.replaceState({}, "", url.toString())
  }, 100)

  /**
   * Set multiple fields at once
   * Updates both state and URL
   * @param value Partial state object with fields to update
   */
  set = (value: Partial<FormFields>) => {
    // Create a new cleaned state with updated values
    const newParams = this.cleanState({
      source: this.source,
      destination: this.destination,
      asset: this.asset,
      receiver: this.receiver,
      sender: this.sender,
      amount: this.amount,
      quoteToken: this.quoteToken,
      ...value,
    })

    // Update state properties
    this.source = newParams.source
    this.destination = newParams.destination
    this.asset = newParams.asset
    this.receiver = newParams.receiver
    this.sender = newParams.sender
    this.amount = newParams.amount
    this.quoteToken = newParams.quoteToken

    // Debounced URL update
    this.debouncedUpdateUrl(newParams)
  }

  /**
   * Update a single field with cascade effects
   * Some fields may reset other dependent fields when changed
   * @param field The field key to update
   * @param valueOrEvent The new value or an event containing the value
   */
  updateField = (field: keyof FormFields, valueOrEvent: string | Event | null) => {
    if (!field) {
      return
    }

    // Extract value from event if needed
    const value = valueOrEvent instanceof Event
      ? (valueOrEvent.target as HTMLInputElement).value
      : valueOrEvent

    // Start with current state
    const currentState = {
      source: this.source,
      destination: this.destination,
      asset: this.asset,
      receiver: this.receiver,
      sender: this.sender,
      amount: this.amount,
      quoteToken: this.quoteToken,
    }

    // Set the new field value
    const updatedState = { ...currentState, [field]: value }

    // Handle dependent field resets
    const resetMapping: Partial<Record<keyof FormFields, Array<keyof FormFields>>> = {
      source: ["asset", "amount", "destination", "quoteToken"],
      asset: ["amount", "destination", "quoteToken"],
      destination: ["receiver", "quoteToken"],
    } as const

    const fieldsToReset = resetMapping[field]
    if (fieldsToReset) {
      fieldsToReset.forEach(resetField => {
        updatedState[resetField] = ""
      })
    }

    // Clean the state
    const newParams = this.cleanState(updatedState)

    // Update state properties
    this.source = newParams.source
    this.destination = newParams.destination
    this.asset = newParams.asset
    this.receiver = newParams.receiver
    this.sender = newParams.sender
    this.amount = newParams.amount
    this.quoteToken = newParams.quoteToken

    // Debounced URL update
    this.debouncedUpdateUrl(newParams)
  }

  reset = () => {
    this.set({
      source: "",
      destination: "",
      asset: "",
      receiver: "",
      sender: "",
      amount: "",
      quoteToken: "",
    })
  }

  /**
   * Flip source and destination chains along with their associated assets
   * This method updates all fields at once without triggering the reset mapping
   * @param newAsset The new asset denom to use after flipping
   */
  flip = (newAsset: Token.Any) => {
    // Store current values
    const currentSource = this.source
    const currentDestination = this.destination

    // Update all fields at once using setWithoutUrlUpdate to avoid reset mapping
    this.setWithoutUrlUpdate({
      source: currentDestination,
      destination: currentSource,
      asset: newAsset.address,
      receiver: "",
      sender: "",
      quoteToken: "",
    })

    // Update URL after all changes are made
    this.debouncedUpdateUrl({
      source: this.source,
      destination: this.destination,
      asset: this.asset,
      receiver: this.receiver,
      sender: this.sender,
      amount: this.amount,
      quoteToken: this.quoteToken,
    })
  }
}
