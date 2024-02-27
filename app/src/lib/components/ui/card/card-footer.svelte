<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements'
  import { cn } from '$lib/utilities/shadcn.ts'

  type $$Props = HTMLAttributes<HTMLDivElement>

  let className: $$Props['class']
  export { className as class }
</script>

<div
  class={cn('flex items-center p-6 pt-0', className)}
  {...$$restProps}
>
  <slot />
</div>
