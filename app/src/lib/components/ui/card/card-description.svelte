<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements'
  import { cn } from '$lib/utilities/shadcn.ts'

  type $$Props = HTMLAttributes<HTMLParagraphElement>

  let className: $$Props['class']
  export { className as class }
</script>

<p
  class={cn('text-sm text-muted-foreground', className)}
  {...$$restProps}
>
  <slot />
</p>
