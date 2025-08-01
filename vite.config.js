import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			"/api": "http://127.0.0.1:3000",
			"/cached": "http://127.0.0.1:3000",
		}
	}
});
