export type SigningMode = "single" | "multi"

const createSigningModeStore = () => {
  let mode = $state<SigningMode>("single")

  return {
    setMode(v: SigningMode) {
      mode = v
    },
    get mode() {
      return mode
    },
  }
}

export const signingMode = createSigningModeStore()
