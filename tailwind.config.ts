import './plugins/tw-css-plugin'

import path, { resolve } from "node:path"
import { withTV } from 'tailwind-variants/transformer'
import type { Config } from 'tailwindcss'
import { theme } from "tailwindcss/defaultConfig"
import plugin from "tailwindcss/plugin"


const twConfig: Config = {
  darkMode: ["class", '[data-theme="dark"]'],
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
    "./src/styles/tailwindcss.css",
  ],
  safelist: [
    "line-clamp-[1]",
    "line-clamp-[2]",
    "line-clamp-[3]",
    "line-clamp-[4]",
    "line-clamp-[5]",
    "line-clamp-[6]",
    "line-clamp-[7]",
    "line-clamp-[8]",
  ],
  prefix: "",
  theme: {
    extend: {
      cursor: {
        button: "var(--cursor-button)",
        select: "var(--cursor-select)",
        checkbox: "var(--cursor-checkbox)",
        link: "var(--cursor-link)",
        menu: "var(--cursor-menu)",
        radio: "var(--cursor-radio)",
        switch: "var(--cursor-switch)",
        card: "var(--cursor-card)",
      },

      width: {
        "feed-col": "var(--fo-feed-col-w)",
      },
      spacing: {
        "safe-inset-top": "var(--fo-window-padding-top, 0)",
        "margin-macos-traffic-light-x": "var(--fo-macos-traffic-light-width, 0)",
        "margin-macos-traffic-light-y": "var(--fo-macos-traffic-light-height, 0)",
      },

      height: {
        screen: "100svh",
      },
      keyframes: {
        "caret-blink": {
          "0%,70%,100%": { opacity: "1" },
          "20%,50%": { opacity: "0" },
        },
        glow: {
          "0%, 100%": { opacity: "0.5" },
          "50%": { opacity: "0.7" },
        },
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        "gradient-x": {
          "0%, 100%": {
            backgroundPosition: "0% 50%",
          },
          "50%": {
            backgroundPosition: "100% 50%",
          },
        },
      },
      animation: {
        "caret-blink": "caret-blink 1.25s ease-out infinite",
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "gradient-x": "gradient-x 3s linear infinite",
        glow: "glow 1.5s ease-in-out infinite",
      },
      fontFamily: {
        theme: "var(--fo-font-family)",
        default: "SN pro, sans-serif, system-ui",
      },

      colors: {
        sidebar: "hsl(var(--fo-sidebar) / <alpha-value>)",
        // Map to actual token variables defined in CSS (@layer base)
        border: "rgb(from var(--color-border) r g b / <alpha-value>)",
        background: "rgb(from var(--color-bg) r g b / <alpha-value>)",
        // Text token mapping so classes like `text-text` and `text-text-secondary` work
        text: {
          DEFAULT: "rgb(from var(--text-primary) r g b / <alpha-value>)",
          secondary: "rgb(from var(--text-secondary) r g b / <alpha-value>)",
          tertiary: "rgb(from var(--text-tertiary) r g b / <alpha-value>)",
          disabled: "rgb(from var(--text-disabled) r g b / <alpha-value>)",
        },

        accent: "hsl(var(--fo-a) / <alpha-value>)",

        theme: {
          // https://uicolors.app/create
          accent: {
            DEFAULT: "hsl(var(--fo-a) / <alpha-value>)",
            "50": "#fff8ed",
            "100": "#ffeed4",
            "200": "#ffdaa8",
            "300": "#ffbe70",
            "400": "#ff9737",
            "500": "#ff760a",
            "600": "#f05d06",
            "700": "#c74507",
            "800": "#9e360e",
            "900": "#7f2f0f",
            "950": "#451505",
          },

          boxShadow: {
            "context-menu":
              "0px 0px 1px rgba(0, 0, 0, 0.4), 0px 0px 1.5px rgba(0, 0, 0, 0.3), 0px 7px 22px rgba(0, 0, 0, 0.25)",
          },

          item: {
            active: "var(--fo-item-active)",
            hover: "var(--fo-item-hover)",
          },
          selection: {
            active: "var(--fo-selection-active)",
            hover: "var(--fo-selection-hover)",
            foreground: "var(--fo-selection-foreground)",
          },

          inactive: "hsl(var(--fo-inactive) / <alpha-value>)",
          disabled: "hsl(var(--fo-disabled) / <alpha-value>)",

          background: "var(--fo-background)",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      backdropBlur: {
        background: "80px",
      },

      typography: (theme) => ({
        zinc: {
          css: {
            "--tw-prose-body": theme("colors.zinc.500"),
            "--tw-prose-quotes": theme("colors.zinc.500"),
          },
        },
      }),
    },
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },

    fontSize: {
      ...theme?.fontSize,
      largeTitle: ["1.625rem", "2rem"], // 26px
      title1: ["1.375rem", "1.625rem"], // 22px
      title2: ["1.0625rem", "1.375rem"], // 17px
      title3: ["0.9375rem", "1.25rem"], // 15px
      headline: ["0.8125rem", "1rem"], // 13px
      body: ["0.8125rem", "1rem"], // 13px
      callout: ["0.75rem", "0.9375rem"], // 12px
      subheadline: ["0.6875rem", "0.875rem"], // 11px
      footnote: ["0.625rem", "0.8125rem"], // 10px
      caption: ["0.625rem", "0.8125rem"], // 10px
    },
  },

  plugins: [
    plugin(({ addUtilities, matchUtilities, theme, addVariant }) => {
      addUtilities({
        ".safe-inset-top": {
          top: "var(--fo-window-padding-top, 0)",
        },
      })

      const safeInsetTopVariants = {}
      for (let i = 1; i <= 16; i++) {
        safeInsetTopVariants[`.safe-inset-top-${i}`] = {
          top: `calc(var(--fo-window-padding-top, 0px) + ${theme(`spacing.${i}`)})`,
        }
      }
      addUtilities(safeInsetTopVariants)

      // left macos traffic light
      const leftMacosTrafficLightVariants = {}
      addUtilities({
        ".left-macos-traffic-light": {
          left: "var(--fo-macos-traffic-light-width, 0)",
        },
      })

      for (let i = 1; i <= 16; i++) {
        leftMacosTrafficLightVariants[`.left-macos-traffic-light-${i}`] = {
          left: `calc(var(--fo-macos-traffic-light-width, 0px) + ${theme(`spacing.${i}`)})`,
        }
      }
      addUtilities(leftMacosTrafficLightVariants)

      // Add arbitrary value support
      matchUtilities(
        {
          "safe-inset-top": (value) => ({
            top: `calc(var(--fo-window-padding-top, 0px) + ${value})`,
          }),
        },
        { values: theme("spacing") },
      )
      addVariant("f-motion-reduce", '[data-motion-reduce="true"] &')
      addVariant("group-motion-reduce", ':merge(.group)[data-motion-reduce="true"] &')
      addVariant("peer-motion-reduce", ':merge(.peer)[data-motion-reduce="true"] ~ &')

      addVariant("zen-mode-macos", ":where(html[data-zen-mode='true'][data-os='macOS']) &")
      addVariant("zen-mode-windows", ":where(html[data-zen-mode='true'][data-os='Windows']) &")

      addVariant("zen-mode", ":where(html[data-zen-mode='true']) &")
      addVariant("macos", ":where(html[data-os='macOS']) &")
      addVariant("windows", ":where(html[data-os='Windows']) &")
    }),
    require("tailwindcss-multi"),
    require("tailwindcss-content-visibility"),
    require("tailwindcss-animate"),
    require("@tailwindcss/container-queries"),
    require("@tailwindcss/typography"),
    require("tailwindcss-motion"),
    require("tailwindcss-safe-area"),
    require(resolve(__dirname, "./src/styles/tailwind-extend.css")),
  ],
}


export default withTV(twConfig)
