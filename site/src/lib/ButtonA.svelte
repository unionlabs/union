<script lang="ts">
  export let href: string;
</script>

<a {href} class="text font-mono font-semibold border-2 border-primary bg-background h-[32px] w-[80px] md:w-[96px] flex items-center justify-center"><slot/></a>
