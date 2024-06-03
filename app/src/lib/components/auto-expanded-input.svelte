<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

let value = ""
let _class = ""
export { _class as class }

function resizeInputOnDynamicContent(node: HTMLInputElement) {
  const measuringElement = document.createElement("div")
  document.body.appendChild(measuringElement)

  function duplicateAndSet() {
    const styles = window.getComputedStyle(node)
    measuringElement.innerHTML = node.value
    Object.assign(measuringElement.style, {
      top: "0",
      height: "0",
      left: "-9999px",
      whiteSpace: "pre",
      overflow: "hidden",
      width: "max-content",
      position: "absolute",
      visibility: "hidden",
      border: styles.border,
      boxSizing: "border-box",
      fontSize: styles.fontSize,
      fontFamily: styles.fontFamily,
      paddingLeft: styles.paddingLeft,
      paddingRight: styles.paddingRight
    })
    node.style.width = `${measuringElement.offsetWidth}px`
  }

  duplicateAndSet()
  /** listen to any style changes */
  const observer = new MutationObserver(duplicateAndSet)
  observer.observe(node, { attributes: true, childList: true, subtree: true })

  node.addEventListener("input", duplicateAndSet)
  return {
    destroy() {
      observer.disconnect()
      node.removeEventListener("input", duplicateAndSet)
    }
  }
}
</script>

<input class={cn('min-w-62', _class)} use:resizeInputOnDynamicContent bind:value {...$$restProps} />
