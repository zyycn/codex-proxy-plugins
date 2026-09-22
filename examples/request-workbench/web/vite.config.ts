import { readFile } from 'node:fs/promises'
import { fileURLToPath, URL } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig(({ mode }) => ({
  resolve: { dedupe: ['vue'] },
  // 页面产物直接在浏览器执行，不能保留库模式下的 Node 环境分支。
  define: {
    'process.env.NODE_ENV': JSON.stringify(mode === 'development' ? 'development' : 'production'),
  },
  plugins: [vue(), tailwindcss(), {
    name: 'plugin-page-entry',
    async generateBundle() {
      this.emitFile({
        type: 'asset',
        fileName: 'index.html',
        source: await readFile(new URL('./index.html', import.meta.url), 'utf8'),
      })
    },
  }],
  build: {
    // 隔离页面只加载包内经典脚本，构建时合并依赖，不在运行时借用宿主模块。
    lib: {
      entry: fileURLToPath(new URL('./src/main.ts', import.meta.url)),
      name: 'RequestWorkbench',
      formats: ['iife'],
      fileName: () => 'app.js',
      cssFileName: 'app',
    },
    cssCodeSplit: false,
    sourcemap: false,
  },
}))
