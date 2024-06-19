<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

let className: string | undefined = undefined

export { className as class }
</script>

<main class={cn(
  className,
  'flex flex-1 flex-col items-center max-h-full gap-6 px-4 pt-6'
)}>
  <slot />
  <div class="md:pb-0 pb-10 text-transparent invisible">.</div>
</main>

<style lang="postcss"></style>
