import type { Action } from "svelte/action"

export const clickOutside: Action<
  HTMLElement,
  undefined,
  {
    onClickOutside: (event: Event) => void
  }
> = node => {
  const handleClick = (event: MouseEvent) => {
    if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
      node.dispatchEvent(new CustomEvent("ClickOutside", { detail: event }))
    }
  }

  document.addEventListener("click", handleClick, true)

  return {
    destroy() {
      document.removeEventListener("click", handleClick, true)
    }
  }
}
