import { writeFileSync } from 'node:fs';

const app = process.argv[2] || 'evo';
const target = process.argv[3] || 'web/js/sentry-config.js';
const fallbackRelease = process.argv[4] || '';

writeFileSync(
  target,
  `window.TRE_STATIC_SENTRY_CONFIG = ${JSON.stringify(
    {
      app,
      dsn: process.env.SENTRY_DSN || '',
      environment: process.env.SENTRY_ENVIRONMENT || 'production',
      release: process.env.SENTRY_RELEASE || process.env.GITHUB_SHA || fallbackRelease,
    },
    null,
    2,
  )};\n`,
);
