<script lang="ts">
let className: string | undefined = "size-12"
export { className as class }
</script>

<div>
  <svg class={className} viewBox="0 0 256 256" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
    <rect x="216" y="144" width="32" height="32" transform="rotate(-180 216 144)" fill="currentColor" class="square square-1"/>
    <rect x="136" y="112" width="32" height="32" fill="currentColor" class="square square-2"/>
    <rect x="120" y="144" width="32" height="32" transform="rotate(-180 120 144)" fill="currentColor" class="square square-3"/>
    <rect x="40" y="112" width="32" height="32" fill="currentColor" class="square square-4"/>
  </svg>
</div>

<style lang="postcss">
    @keyframes fadeInOut {
        0%, 100% { opacity: 0; }
        75% { opacity: 1; }
    }

    .square {
        opacity: 0;
        animation: fadeInOut 700ms infinite reverse;
    }

    .square-1 { animation-delay: 400ms; }
    .square-2 { animation-delay: 300ms; }
    .square-3 { animation-delay: 200ms; }
    .square-4 { animation-delay: 100ms; }
</style>
