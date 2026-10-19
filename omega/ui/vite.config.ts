import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      // omega-server (W7 inc2) : API typée du harnais.
      '/api': 'http://localhost:8787',
    },
  },
});
