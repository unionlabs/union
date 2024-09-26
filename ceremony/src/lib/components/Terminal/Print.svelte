<script lang="ts">
  let {children, class: className = "", ...rest} = $props()
</script>

<p class="w-fit text-neutral-300 text-sm sm:text-base {className}" {...rest}>{@render children()}</p>