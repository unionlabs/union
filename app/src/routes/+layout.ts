import { setMode } from "mode-watcher"

/** @see https://kit.svelte.dev/docs/page-options */
export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

const load = () => {
  setMode("light")
}
