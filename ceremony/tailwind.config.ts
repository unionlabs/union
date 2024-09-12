import type { Config } from "tailwindcss"
import defaultTheme from "tailwindcss/defaultTheme"

export default (<Config>{
  content: ["./src/**/*.{html,js,svelte,ts}"],
  safelist: ["dark", "light"],
  darkMode: "class",
  theme: {
    fontFamily: _ => ({
      gunship: ["gunship", ...defaultTheme.fontFamily.sans],
      supermolot: ["tt-supermolot-neue", ...defaultTheme.fontFamily.sans],
      sans: ["jetbrains-mono", ...defaultTheme.fontFamily.sans],
      mono: ["jetbrains-mono", ...defaultTheme.fontFamily.mono]
    }),
    extend: {
      colors: {
        union: {
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
          text: {
            primary: "#FFFFFF"
          },
          heading: {
            primary: "#FFFFFF"
          },
          background: {
            primary: "#000000",
            secondary: "#1C1C1C",
            black: "#000000",
            white: "#FFFFFF"
          }
        }
      }
    }
  },
  plugins: []
})
