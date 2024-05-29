<script>
    import { cn } from "$lib/utilities/shadcn";
</script>
<div class="flex flex-row h-full flex-1">
  <nav class={cn("flex flex-col border-solid border-r border-red-500 w-40")}>
    <a href="/explorer/blocks">blocks</a>
    <a href="/explorer/connections">connections</a>
  </nav>

  <div class="flex-1">
    <slot/>
  </div>
</div>
