<script lang="ts">
  import { cn } from '$lib/utilities/shadcn.ts'
  import type { HTMLAttributes } from 'svelte/elements'

  type $$Props = HTMLAttributes<HTMLDivElement>

  let className: $$Props['class']
  export { className as class }
</script>

<div
  class={cn('p-6 pt-0', className)}
  {...$$restProps}
>
  <slot />
</div>
