<script lang="ts">
import { readable } from "svelte/store"
import { relativeTime } from "$lib/utilities/format.ts"

  interface Props {
    timestamp: Date;
    class: string;
  }

  let { timestamp, class: _class }: Props = $props();


const time = readable(timestamp, set => {
  const interval = setInterval(() => set(timestamp), 1_000)
  return () => clearInterval(interval)
})
</script>

<time class={_class}>{relativeTime({ timestamp: $time })}</time>
