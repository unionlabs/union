<script lang="ts">
import { readable } from "svelte/store"
import { relativeTime } from "$lib/utilities/format.ts"

export let timestamp: Date

const time = readable(timestamp, set => {
  const interval = setInterval(() => set(timestamp), 1_000)
  return () => clearInterval(interval)
})
</script>

<time>{relativeTime({ timestamp: $time })}</time>
