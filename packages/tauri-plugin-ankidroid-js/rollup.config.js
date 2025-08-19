import { nodeResolve } from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';

export default {
  input: 'src/index.ts',
  output: [
    {
      file: 'dist/index.js',
      format: 'es',
    },
  ],
  plugins: [
    nodeResolve({
      preferBuiltins: false,
    }),
    typescript({
      declaration: true,
      declarationDir: 'dist',
      rootDir: 'src',
    }),
  ],
  external: ['@tauri-apps/api/core'],
};
