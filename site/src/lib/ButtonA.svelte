<script lang="ts">
  export let href: string;
</script>

<a {href} class="text font-mono font-semibold border-2 border-primary bg-background h-[32px] w-[144px] md:w-[160px] flex items-center justify-center"><slot/></a>
