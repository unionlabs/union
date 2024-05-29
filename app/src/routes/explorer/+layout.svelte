<script>
    import { cn } from "$lib/utilities/shadcn";
</script>

<main class="flex flex-row flex-1 overflow-y-hidden">
  <nav class={cn("flex flex-col border-solid border-r border-red-500 w-40")}>
    <a href="/explorer/blocks">blocks</a>
    <a href="/explorer/connections">connections</a>
  </nav>

  <div class="flex-1 overflow-y-scroll">
    <slot/>
  </div>
</main>
