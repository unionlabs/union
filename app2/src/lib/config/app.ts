import base64Icon from "./base64.txt?raw"
import { uiStore } from "$lib/stores/ui.svelte.ts"

export const TESTNET_APP_INFO = {
  base64Icon,
  name: "Union",
  baseUrl: `https://${uiStore.activeEdition}.union.build`,
  docs: "https://docs.union.build",
  iconUrl: "https://app.union.build/images/logo.png"
}
