import type { Action } from "svelte/action"

export const portal: Action<HTMLElement, boolean> = (node, persistent) => {
  if (!persistent) {
    document.querySelector("#modal-container")?.appendChild(node).focus()
  }
}
