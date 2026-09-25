import { readFile } from 'node:fs/promises'
import { fileURLToPath, URL } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig(({ mode, command }) => {
  const sourceUi = mode === 'source'
  const uiRoot = new URL('../../../../ui/', import.meta.url)
  return {
    resolve: {
      // 源码联调显式启用；普通开发与发行构建继续使用锁定的包。
      alias: [
        { find: '@', replacement: fileURLToPath(new URL('./src', import.meta.url)) },
        ...(sourceUi
          ? [
              { find: /^@codex-proxy\/ui$/, replacement: fileURLToPath(new URL('src/index.ts', uiRoot)) },
              { find: '@codex-proxy/ui/theme', replacement: fileURLToPath(new URL('src/theme/index.ts', uiRoot)) },
              { find: '@codex-proxy/ui/styles.css', replacement: fileURLToPath(new URL('src/styles/index.css', uiRoot)) },
              { find: '@codex-proxy/ui/tailwind.css', replacement: fileURLToPath(new URL('src/styles/tailwind.css', uiRoot)) },
              { find: /^@codex-proxy\/ui\/(.+)$/, replacement: fileURLToPath(new URL('src/components/$1/index.ts', uiRoot)) },
            ]
          : []),
      ],
      // 与共享 UI 使用同一份图标上下文，确保按钮的默认尺寸能够传给图标。
      dedupe: ['vue', '@lucide/vue'],
    },
    define: {
      'process.env.NODE_ENV': JSON.stringify(command === 'serve' ? 'development' : 'production'),
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
    server: {
      fs: sourceUi ? { allow: [fileURLToPath(new URL('.', import.meta.url)), fileURLToPath(uiRoot)] } : undefined,
    },
    build: {
      outDir: sourceUi ? '.vite/source-dist' : 'dist',
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
  }
})
