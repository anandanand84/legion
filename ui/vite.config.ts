import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import wasmPack from 'vite-plugin-wasm-pack';
const config: UserConfig = {
	plugins: [wasmPack('../../legion'),  sveltekit()]
};

export default config;
