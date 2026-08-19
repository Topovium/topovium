// SPDX-License-Identifier: GPL-3.0-or-later
import type { NextConfig } from 'next';

const config: NextConfig = {
  reactStrictMode: true,

  // The site is static: no server, no database, no account. It hosts anywhere and
  // stays available whatever happens to any single provider.
  output: 'export',
  images: { unoptimized: true },

  typescript: {
    // A type error must fail the build. Shipping a site that does not typecheck is
    // how `any` creeps back in through the side door. Next 16 no longer runs ESLint
    // during `next build`, so `just web-check` runs it as a separate gate.
    ignoreBuildErrors: false,
  },
};

export default config;
