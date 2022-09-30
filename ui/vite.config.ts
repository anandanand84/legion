import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import wasmPack from 'vite-plugin-wasm-pack';
import plainText from 'vite-plugin-plain-text';

console.log(plainText);
const config: UserConfig = {
	plugins: [
		// (plainText as any).default(/\/tests\/test_*/), 
		wasmPack('../../legion'),  sveltekit()
	]
};

export default config;
