<script lang="ts">
type Props = {
  class?: string
}

let { class: className = "size-6" }: Props = $props()
</script>

<svg xmlns="http://www.w3.org/2000/svg" class={className} viewBox="0 0 24 24">
  <path
    fill="none"
    stroke="currentColor"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-width="1.5"
    d="m3 21l7.548-7.548M21 3l-7.548 7.548m0 0L8 3H3l7.548 10.452m2.904-2.904L21 21h-5l-5.452-7.548"
    color="currentColor"
  />
</svg>
