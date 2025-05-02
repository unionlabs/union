type ModifierKey = "cmd" | "ctrl" | "option" | "alt" | "shift"
type KeyCode = `key${string}` | `digit${string}` | `f${number}`
type ShortcutKey = ModifierKey | KeyCode

interface Shortcut {
  keys: Set<string>
  action: () => void
}

export class KeyboardShortcuts {
  private shortcuts: Array<Shortcut> = $state([])

  addShortcut(keys: Array<ShortcutKey>, action: () => void) {
    const normalizedKeys = new Set(keys.map(k => k.toLowerCase()))
    this.shortcuts = [...this.shortcuts, { keys: normalizedKeys, action }]
  }

  removeShortcut(action: () => void) {
    this.shortcuts = this.shortcuts.filter(s => s.action !== action)
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    const pressedKeys = new Set<string>()

    if (event.metaKey) {
      pressedKeys.add("cmd")
    }
    if (event.ctrlKey) {
      pressedKeys.add("ctrl")
    }
    if (event.altKey) {
      pressedKeys.add("option")
    }
    if (event.shiftKey) {
      pressedKeys.add("shift")
    }
    pressedKeys.add(event.code.toLowerCase())

    for (const { keys: expectedKeys, action } of this.shortcuts) {
      if (expectedKeys.size !== pressedKeys.size) {
        continue
      }

      let allMatch = true
      for (const key of expectedKeys) {
        if (!pressedKeys.has(key)) {
          allMatch = false
          break
        }
      }

      if (allMatch) {
        event.preventDefault()
        action()
        break
      }
    }
  }

  constructor() {
    window.addEventListener("keydown", this.handleKeyDown)
  }

  destroy() {
    window.removeEventListener("keydown", this.handleKeyDown)
  }
}

export const keyboardShortcuts = new KeyboardShortcuts()
