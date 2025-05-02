import { xTermTheme } from "#/lib/xterm/theme"
import { ClipboardAddon } from "@xterm/addon-clipboard"
import { FitAddon } from "@xterm/addon-fit"
import { ImageAddon } from "@xterm/addon-image"
import { SerializeAddon } from "@xterm/addon-serialize"
import { WebLinksAddon } from "@xterm/addon-web-links"
import { Terminal } from "@xterm/xterm"

export type XTermAddon = {
  fitAddon: FitAddon
  imageAddon: ImageAddon
  webLinksAddon: WebLinksAddon
  clipboardAddon: ClipboardAddon
  serializeAddon: SerializeAddon
}
export type { Terminal }

export function initiateTerminal(
  terminalElement: HTMLElement,
  options: {
    fontSize?: number
    readonly?: boolean
  } = { fontSize: 28, readonly: false },
): {
  terminal: Terminal
  addons: XTermAddon
} {
  const terminal = new Terminal({
    convertEol: true,
    cursorBlink: true,
    theme: xTermTheme,
    cursorStyle: "bar",
    windowOptions: {},
    allowProposedApi: true,
    cursorInactiveStyle: "bar",
    fontSize: options.fontSize,
    disableStdin: options.readonly,
    drawBoldTextInBrightColors: true,
  })
  const fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)

  const serializeAddon = new SerializeAddon()
  terminal.loadAddon(serializeAddon)

  const webLinksAddon = new WebLinksAddon()
  terminal.loadAddon(webLinksAddon)

  const clipboardAddon = new ClipboardAddon()
  terminal.loadAddon(clipboardAddon)

  const imageAddon = new ImageAddon()
  terminal.loadAddon(imageAddon)
  terminal.open(terminalElement)
  fitAddon.fit()

  return {
    terminal,
    addons: {
      fitAddon,
      imageAddon,
      webLinksAddon,
      clipboardAddon,
      serializeAddon,
    },
  }
}
