<script lang="ts">
import { cn } from '$/lib/utilities/shadcn'
import { Badge } from '$lib/components/ui/badge'

export let className = ''
</script>

<Badge
  variant="outline"
  class={cn(
    'text-sm text-left left-2 top-2 absolute border-solid border-amber-400/40 rounded-md',
    className,
  )}
>
  page under construction 👷‍♂️
</Badge>
