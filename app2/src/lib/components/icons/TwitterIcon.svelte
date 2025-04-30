<script lang="ts">
import { cn } from "$lib/utils"

interface Props {
  class?: string
  size?: number
}

const { class: className = "", size = 16 }: Props = $props()
</script>

<svg 
  xmlns="http://www.w3.org/2000/svg" 
  width={size} 
  height={size} 
  viewBox="0 0 24 24" 
  fill="none" 
  stroke="currentColor" 
  stroke-width="2" 
  stroke-linecap="round" 
  stroke-linejoin="round" 
  class={cn("inline-block", className)}
>
  <path d="M6 4l12 16M6 20L18 4" />
</svg>
