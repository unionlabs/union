<script lang="ts">
import { cn } from "#/lib/shadcn.ts"
import GraphqlIcon from "icons:svelte/logos/graphql"

interface Props {
  class: string
  query: string
}

export const { query, class: className }: Props = $props()
</script>

<a
  target="_blank"
  rel="noopener noreferrer"
  title="Open in GraphQL playground"
  href={`/reference/graphql?query=${encodeURIComponent(query)}`}
  class={cn(className, 'size-10 my-auto hover:bg-muted/10 rounded-sm p-1 hover:cursor-pointer')}
>
  <GraphqlIcon class='m-auto size-full' />
</a>

<style lang="postcss"></style>
