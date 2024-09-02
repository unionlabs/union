import type { Config } from "tailwindcss"
import defaultTheme from "tailwindcss/defaultTheme"

export default (<Config>{
  content: ['./src/**/*.{html,js,svelte,ts}'],
  safelist: ["dark", "light"],
  darkMode: "class",
  theme: {
    extend: {
      fontFamily: _ => ({
        gunship: ["gunship", ...defaultTheme.fontFamily.sans],
        supermolot: ["tt-supermolot-neue", ...defaultTheme.fontFamily.sans],
        sans: ["jetbrains-mono", ...defaultTheme.fontFamily.sans],
        mono: ["jetbrains-mono", ...defaultTheme.fontFamily.mono]
      }),
      colors: {
        accent: {
          50: "#FAFEFF",
          100: "#F0FCFF",
          200: "#DCF8FE",
          300: "#C8F4FE",
          400: "#B4F0FD",
          500: "#A0ECFD",
          600: "#5FDFFC",
          700: "#1ED2FA",
          800: "#04ACD2",
          900: "#037791",
          950: "#025C70"
        },
        background: {
          "light": "#b9bec6",
          "light-secondary": "#a7aaae"
        }
      },
    },
  },
  plugins: [],
})

