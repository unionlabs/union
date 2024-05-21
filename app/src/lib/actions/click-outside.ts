import type { Action } from "svelte/action"

type ClickOutsideParams = () => void

export const clickOutside: Action<HTMLElement, ClickOutsideParams> = (node, callback) => {
  const clickHandler = (event: MouseEvent) => {
    if (node.contains(event.target as Node)) return
    callback?.()
  }
  document.addEventListener("click", clickHandler)
  return {
    destroy: () => {
      document.removeEventListener("click", clickHandler)
    }
  }
}
