<script lang="ts">
let { children, class: className = "", focus = undefined, value = $bindable(), ...rest } = $props()
</script>

<button
        bind:this={value}
        class="px-2 block outline-none focus:ring-transparent focus:border-none text-sm sm:text-base {className}"
        class:terminal-blink={focus}
        {...rest}>
  {@render children()}
</button>

<style>
    .terminal-blink {
        animation: blink 1s step-end infinite;
    }

    @keyframes blink {
        0%, 100% {
            background-color: #A0ECFD;
            color: black;
        }
        50% {
            background-color: transparent;
            color: white;
        }
    }
</style>