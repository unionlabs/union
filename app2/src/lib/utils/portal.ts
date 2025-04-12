import type { Action } from "svelte/action"

export const portal: Action<HTMLElement> = node => {
  document.querySelector("#modal-container")?.appendChild(node).focus()
}
