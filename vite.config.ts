import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import jotaiDebugLabel from "jotai/babel/plugin-debug-label";
import jotaiReactRefresh from "jotai/babel/plugin-react-refresh";
import lightningcss from "vite-plugin-lightningcss";
import svgr from "vite-plugin-svgr";
import wasm from "vite-plugin-wasm";
import { resolve } from "path";
import path from 'path'
import { astPlugin } from './plugins/vite-ast'

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
	build: {
		target:
			process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari15",
		minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
		modulePreload: {
			polyfill: false,
		},
		rollupOptions: {
			shimMissingExports: true,
			input: {
				index: resolve(__dirname, "index.html"),
			},
		},
		sourcemap: "inline",
	},
	plugins: [
		react({
			babel: {
				plugins: [jotaiDebugLabel, jotaiReactRefresh],
			},
		}),
		wasm(),
		// topLevelAwait(),

		svgr({
			svgrOptions: {
				ref: true,
			},
			include: ["./src/**/*.svg?react", "../react-full/src/**/*.svg?react"],
		}),
		lightningcss({
			browserslist: "safari >= 10.13, chrome >= 91",
		}),
		astPlugin,
	],
	resolve: {
		alias: {
          '~': path.resolve(__dirname, './src'),
          '@locales': path.resolve(__dirname, './locales'),
        },
	},
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 5173,
		host: host || false,
		strictPort: true,
		warmup: {
			clientFiles: [
				"src/**/*.tsx",
				"src/**/*.ts",
				"src/**/*.css",
				"src/**/*.svg?react",
			],
		},
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 1421,
				}
			: undefined,
	},
	// 3. to make use of `TAURI_DEBUG` and other env variables
	// https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
	envPrefix: ["VITE_", "TAURI_"],
});
