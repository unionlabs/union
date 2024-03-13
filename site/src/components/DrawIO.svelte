<script lang="ts">
import { onMount } from "svelte"

export let src: string

const htmlElement = document.querySelector("html")
const currentTheme = htmlElement?.getAttribute("data-theme")
const selectThemeElement = document.querySelector("select")

let filter = currentTheme === "dark" ? "invert(1)" : "none"

function onThemeChange(event: Event) {
  const selectedTheme = event?.target?.value
  filter = selectedTheme === "dark" ? "invert(1)" : "none"
}

onMount(() => {
  selectThemeElement?.addEventListener("change", onThemeChange)
  return () => {
    selectThemeElement?.removeEventListener("change", onThemeChange)
  }
})
</script>

<div style:filter class="mt-8">
  {@html src}
</div>
