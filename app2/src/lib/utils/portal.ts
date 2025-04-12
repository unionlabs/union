import type { Action } from "svelte/action"

export const portal: Action<HTMLElement> = node => {
  document.querySelector("body")?.appendChild(node).focus()
}
