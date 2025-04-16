import { debounce } from "$lib/utils"

/**
 * Type definition for form fields managed by this store
 */
export type FormFields = {
  source: string // Source identifier
  destination: string // Destination identifier
  asset: string // Asset type or identifier
  receiver: string // Receiver of the transaction/action
  amount: string // Amount value (stored as string for form handling)
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
export class RawTransferSvelte {
  // Reactive state fields using Svelte 5 $state syntax
  source: string = $state("")
  destination: string = $state("")
  asset: string = $state("")
  receiver: string = $state("")
  amount: string = $state("")

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
      const initialValues: Partial<FormFields> = {}

      // Get each field from URL parameters
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
    const { source = "", destination = "", asset = "", receiver = "", amount = "" } = state
    return { source, destination, asset, receiver, amount }
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
      amount: this.amount,
      ...value
    })

    // Update state properties
    this.source = newParams.source
    this.destination = newParams.destination
    this.asset = newParams.asset
    this.receiver = newParams.receiver
    this.amount = newParams.amount
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
      amount: this.amount,
      ...value
    })

    // Update state properties
    this.source = newParams.source
    this.destination = newParams.destination
    this.asset = newParams.asset
    this.receiver = newParams.receiver
    this.amount = newParams.amount

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
    if (!field) return

    // Extract value from event if needed
    const value =
      valueOrEvent instanceof Event ? (valueOrEvent.target as HTMLInputElement).value : valueOrEvent

    // Start with current state
    const currentState = {
      source: this.source,
      destination: this.destination,
      asset: this.asset,
      receiver: this.receiver,
      amount: this.amount
    }

    // Set the new field value
    const updatedState = { ...currentState, [field]: value }

    // Handle dependent field resets
    const resetMapping: Partial<Record<keyof FormFields, Array<keyof FormFields>>> = {
      source: ["asset", "amount"],
      asset: ["amount"],
      destination: ["receiver"]
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
    this.amount = newParams.amount

    // Debounced URL update
    this.debouncedUpdateUrl(newParams)
  }

  reset = () => {
    this.set({
      source: "",
      destination: "",
      asset: "",
      receiver: "",
      amount: ""
    })
  }
}
