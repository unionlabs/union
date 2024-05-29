import type { ActionReturn } from "svelte/action"
import { elementHasFocus } from "$lib/utilities/index.ts"

type TextResolver<K extends keyof HTMLElementEventMap> = (input: {
  node: HTMLElement
  trigger: HTMLElement
  event: HTMLElementEventMap[K]
}) => string | Promise<string>

interface CopyConfig<K extends keyof HTMLElementEventMap> {
  enabled: boolean
  synthetic: boolean
  event: K | Array<K>
  trigger: HTMLElement
  text: string | TextResolver<K>
}

type CopyParameter<K extends keyof HTMLElementEventMap> = Partial<CopyConfig<K>> | undefined

interface CopyAttributes {
  "on:copied"?: (event: CustomEvent<{ text: string }>) => void
}

type CopyReturn<K extends keyof HTMLElementEventMap> = ActionReturn<
  CopyParameter<K>,
  CopyAttributes
>

const resolveConfiguration = <K extends keyof HTMLElementEventMap>(
  node: HTMLElement,
  parameters: CopyParameter<K> = {}
): {
  trigger: HTMLElement
  text: TextResolver<K>
  events: K | Array<K>
  enabled: boolean
  synthetic: boolean
} => {
  const { trigger = node, enabled = true, synthetic = false } = parameters
  const text = (
    typeof parameters.text === "function"
      ? parameters.text
      : () => parameters.text ?? node.textContent
  ) as TextResolver<K>
  const events =
    typeof parameters.event === "string"
      ? [parameters.event]
      : parameters.event ?? (["click"] as K | Array<K>)
  return { trigger, enabled, text, events, synthetic }
}

export function copyTextAction<K extends keyof HTMLElementEventMap>(
  node: HTMLElement,
  parameters: CopyParameter<K>
): CopyReturn<K> {
  let { trigger, text, events, enabled, synthetic } = resolveConfiguration(node, parameters)

  const handle = async (event: HTMLElementEventMap[K]) => {
    console.log(event)
    const _text = await text({ node, trigger, event })
    copyToClipboard(_text)
    const detail = { text: _text }
    node.dispatchEvent(new CustomEvent("copied", { detail }))
    if (!synthetic) return
    const clipboardData = new DataTransfer()
    clipboardData.setData("text/plain", _text)
    const clipboardEvent = new ClipboardEvent("copy", { clipboardData })
    node.dispatchEvent(clipboardEvent)
  }

  const addEvents = () => {
    if (trigger) {
      for (const event of events) node.addEventListener(event as K, handle)
    }
  }

  const removeEvents = () => {
    if (trigger) {
      for (const event of events) node.removeEventListener(event as K, handle)
    }
  }

  if (enabled) addEvents()

  return {
    update: () => [removeEvents(), addEvents()],
    destroy: () => removeEvents()
  }
}

/**
 * Copied from {@link https://github.com/vuejs/vitepress/blob/43c89d66c0d8c87e244a0a0e73a897509b977e65/src/client/app/composables/copyCode.ts#L3}
 */
function copyToClipboard(text: string): Promise<void> | undefined {
  try {
    return navigator.clipboard.writeText(text)
  } catch {
    const element = document.createElement("textarea")

    const previouslyFocusedElement = document.activeElement

    element.value = text
    // Prevent keyboard from showing on mobile
    element.setAttribute("readonly", "")
    Object.assign(element.style, {
      left: "-9999px",
      fontSize: "12pt", // Prevent zooming on iOS
      contain: "strict",
      position: "absolute"
    })

    const selection = document.getSelection()
    const originalRange = selection ? selection.rangeCount > 0 && selection.getRangeAt(0) : null

    document.body.appendChild(element)
    element.select()

    // Explicit selection workaround for iOS
    element.selectionStart = 0
    element.selectionEnd = text.length

    document.execCommand("copy")
    document.body.removeChild(element)

    if (originalRange && selection) {
      selection.removeAllRanges()
      selection.addRange(originalRange)
    }

    if (!previouslyFocusedElement) return
    if (!elementHasFocus(previouslyFocusedElement)) return
    previouslyFocusedElement.focus()
  }
}
