import type { Config } from "tailwindcss"

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],

  theme: {
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
} satisfies Config
