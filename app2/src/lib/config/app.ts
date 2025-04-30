import base64Icon from "./base64.txt?raw";
import { uiStore } from "$lib/stores/ui.svelte.ts";

export function getTestnetAppInfo() {
  const edition = uiStore.activeEdition;

  return {
    base64Icon,
    name: "Union",
    baseUrl: `https://${edition}.union.build`,
    docs: "https://docs.union.build",
    iconUrl: "https://app.union.build/images/logo.png",
  };
}
