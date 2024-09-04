<script lang="ts">
  let { children, class: CLASS, ...props } =$props()
</script>

<h6 class={`text-union-heading-primary text-base font-bold font-supermolot mb-1.5 ${CLASS}`} {...props}>
  {@render children()}
</h6>
