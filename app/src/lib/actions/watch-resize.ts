import type { Action } from "svelte/action"

type WatchResizeParams = () => void

/**
 *   let windowSize = { width: window.innerWidth, height: window.innerHeight }
  const handleResize = () => (windowSize = { width: window.innerWidth, height: window.innerHeight })

  onMount(() => {
    window.addEventListener('resize', handleResize)
    return () => {
      window.removeEventListener('resize', handleResize)
    }
  })

 */

export const watchResize: Action<HTMLElement, WatchResizeParams> = (node, callback) => {
  console.log("watchResize", node, callback)
  const clickHandler = (event: MouseEvent) => {
    if (node.contains(event.target as Node)) return
    callback?.()
  }

  // document.addEventListener("click", clickHandler)
  // return {
  //   destroy: () => {
  //     document.removeEventListener("click", clickHandler)
  //   }
  // }
}
