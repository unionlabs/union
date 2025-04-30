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
  <circle cx="9" cy="12" r="1" />
  <circle cx="15" cy="12" r="1" />
  <path d="M7.5 7.5c3.5-1 5.5-1 9 0" />
  <path d="M7.5 16.5c3.5 1 5.5 1 9 0" />
  <path d="M15.5 17c0 1 1.5 3 2 3 1.5 0 2.833-1.667 3.5-3 .667-1.667.5-5.833-1.5-11.5-1.457-1.015-3-1.34-4.5-1.5l-1 2.5" />
  <path d="M8.5 17c0 1-1.356 3-1.832 3-1.429 0-2.698-1.667-3.333-3-.635-1.667-.48-5.833 1.428-11.5C6.151 4.485 7.545 4.16 9 4l1 2.5" />
</svg>
