<script lang="ts">
import { cn } from "$lib/utils"

type Direction = "top" | "bottom"

let {
  direction = "bottom",
  class: className = "",
}: {
  direction?: Direction
  class?: string
} = $props()

const gradientDirection = $derived(direction === "top" ? "0deg" : "180deg")
const positionClass = $derived(direction === "top" ? "top-[-100px]" : "bottom-[-40px]")
</script>

<div class={cn("w-full h-full overflow-hidden", className)}>
  <div
    style={`background: conic-gradient(from ${gradientDirection} at 50% 50%, #4bb7c3 0deg, #000 80deg, #000 280deg, #4bb7c3 1turn); filter: blur(85px); opacity: 40%;`}
    class={`w-[140%] h-[100%] relative ${positionClass} -translate-x-[15%]`}
  >
  </div>
</div>
