<script lang="ts">
  let {children, class: CLASS, ...props} = $props()
</script>

<h1 class={`text-union-text-primary text-4xl font-bold font-supermolot uppercase  ${CLASS}`} {...props}>
  {@render children()}
</h1>
