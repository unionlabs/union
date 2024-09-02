import type { Config } from "tailwindcss"
import defaultTheme from "tailwindcss/defaultTheme"

export default (<Config>{
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      fontFamily: _ => ({
        gunship: ["gunship", ...defaultTheme.fontFamily.sans],
        supermolot: ["tt-supermolot-neue", ...defaultTheme.fontFamily.sans],
        sans: ["jetbrains-mono", ...defaultTheme.fontFamily.sans],
        mono: ["jetbrains-mono", ...defaultTheme.fontFamily.mono]
      }),
    },
  },
  plugins: [],
})

