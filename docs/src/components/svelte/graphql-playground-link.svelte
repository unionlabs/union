<script lang="ts">
import { cn } from "#/lib/shadcn.ts"
import graphqlSvg from "#/assets/icons/graphql.svg?raw"

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
  class={cn(className, 'size-12 my-auto hover:bg-muted/10 rounded-sm p-1 hover:cursor-pointer')}
>
  {@html graphqlSvg}
</a>

<style lang="postcss"></style>
