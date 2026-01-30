import { browser } from "$app/environment"

type AddressFormat = "hex" | "base64"

const STORAGE_KEY = "address-format"

// Get initial value from localStorage or default to hex
function getInitialFormat(): AddressFormat {
  if (!browser) {
    return "hex"
  }
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === "base64" || stored === "hex") {
    return stored
  }
  return "hex"
}

let format = $state<AddressFormat>(getInitialFormat())

export const addressFormat = {
  get value() {
    return format
  },
  set value(newFormat: AddressFormat) {
    format = newFormat
    if (browser) {
      localStorage.setItem(STORAGE_KEY, newFormat)
    }
  },
  toggle() {
    this.value = format === "hex" ? "base64" : "hex"
  },
  isHex() {
    return format === "hex"
  },
  isBase64() {
    return format === "base64"
  },
}
