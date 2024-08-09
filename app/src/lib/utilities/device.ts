import { writable } from "svelte/store"

export const deviceWidth = writable<number>()

export const supportsWebGL = writable<boolean>(false)

export function checkWebGLSupport() {
  try {
    const canvas = document.createElement("canvas")
    const gl = canvas.getContext("webgl") || canvas.getContext("experimental-webgl")
    supportsWebGL.set(!!gl)
  } catch (e) {
    supportsWebGL.set(false)
  }
}
