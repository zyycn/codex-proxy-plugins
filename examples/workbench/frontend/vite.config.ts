import { readFile } from 'node:fs/promises'
import { fileURLToPath, URL } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig(({ mode }) => ({
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
    // 与共享 UI 使用同一份图标上下文，确保按钮的默认尺寸能够传给图标。
    dedupe: ['vue', '@lucide/vue'],
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(mode === 'development' ? 'development' : 'production'),
  },
  plugins: [
    vue(),
    tailwindcss(),
    {
      name: 'plugin-page-entry',
      async generateBundle() {
        const template = await readFile(new URL('./index.html', import.meta.url), 'utf8')
        this.emitFile({
          type: 'asset',
          fileName: 'index.html',
          source: template
            .replace('</head>', '    <link rel="stylesheet" href="./app.css" />\n  </head>')
            .replace('<script type="module" src="/src/main.ts"></script>', '<script src="./app.js" defer></script>'),
        })
      },
    },
  ],
  build: {
    lib: {
      entry: fileURLToPath(new URL('./src/main.ts', import.meta.url)),
      name: 'Workbench',
      formats: ['iife'],
      fileName: () => 'app.js',
      cssFileName: 'app',
    },
    cssCodeSplit: false,
    sourcemap: false,
  },
}))
