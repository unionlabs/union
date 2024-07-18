import { toast } from "svelte-sonner"

export function copy(node: HTMLElement, text?: string | number) {
  node.classList.add("cursor-pointer")

  const handleClick = async () => {
    const textToCopy = (
      text !== undefined
        ? text.toString()
        : node.innerText ||
          (node as HTMLInputElement).value ||
          node.getAttribute("data-copy-text") ||
          ""
    ).toString()
    try {
      await navigator.clipboard.writeText(textToCopy)
      toast.info("Copied!")
    } catch (err) {
      toast.info("Failed to copy")
    }
  }

  node.addEventListener("click", handleClick)

  return {
    update(newText?: string | number) {
      text = newText
    },
    destroy() {
      node.removeEventListener("click", handleClick)
    }
  }
}
