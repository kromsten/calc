import { build } from 'esbuild';
import Fs from 'fs';
import { forEach } from 'ramda';
import path from 'path';

const outdir = './dist';

(async () => {
  try {
    Fs.rmSync(outdir, { recursive: true, force: true });

    const res = await build({
      entryPoints: ['./src/handlers/validate-xdefi-achievements/app.ts', './src/handlers/dummy/app.ts'],
      outdir,
      minify: true,
      bundle: true,
      platform: 'node',
    });

    if (res.warnings.length > 0) {
      console.log(`WARNINGS: ${res.warnings}`);
    }
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
})();
