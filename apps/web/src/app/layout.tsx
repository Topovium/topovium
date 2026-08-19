// SPDX-License-Identifier: GPL-3.0-or-later
import type { Metadata } from 'next';
import { Archivo, IBM_Plex_Mono, IBM_Plex_Sans } from 'next/font/google';
import Link from 'next/link';
import './globals.css';

const archivo = Archivo({
  subsets: ['latin'],
  variable: '--font-archivo',
  axes: ['wdth'],
  display: 'swap',
});

const plexSans = IBM_Plex_Sans({
  subsets: ['latin'],
  weight: ['400', '500', '600'],
  variable: '--font-plex-sans',
  display: 'swap',
});

const plexMono = IBM_Plex_Mono({
  subsets: ['latin'],
  weight: ['400', '500'],
  variable: '--font-plex-mono',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'Topovium',
  description:
    'A 3D creation environment for desktop, tablet, and the browser. Written in Rust. One project file, five platforms, no account required.',
  metadataBase: new URL('https://topovium.org'),
};

const NAV = [
  { href: '/download', label: 'Download' },
  { href: '/features', label: 'Features' },
  { href: '/benchmarks', label: 'Benchmarks' },
  { href: '/roadmap', label: 'Roadmap' },
] as const;

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>): React.JSX.Element {
  return (
    <html lang="en" className={`${archivo.variable} ${plexSans.variable} ${plexMono.variable}`}>
      <body className="min-h-dvh">
        <a
          href="#main"
          className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:bg-near focus:px-3 focus:py-2 focus:font-mono focus:text-xs focus:text-void"
        >
          Skip to content
        </a>

        <div className="mx-auto flex max-w-[88rem]">
          {/* The measurement rail. A plotter margin, not decoration: it is the same
              device the app's own timing HUD uses. */}
          <div aria-hidden className="rail hidden w-10 shrink-0 border-r border-rule lg:block" />

          <div className="min-w-0 flex-1">
            <header className="sticky top-0 z-40 border-b border-rule bg-void/85 backdrop-blur">
              <nav className="flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3 sm:px-8">
                <Link
                  href="/"
                  className="display text-lg text-chalk transition-colors hover:text-near"
                >
                  Topovium
                </Link>
                <span className="eyebrow hidden sm:inline">0.0.1 · pre-alpha</span>
                <ul className="ml-auto flex flex-wrap items-center gap-x-5 gap-y-1">
                  {NAV.map((item) => (
                    <li key={item.href}>
                      <Link
                        href={item.href}
                        className="font-mono text-xs text-mute transition-colors hover:text-chalk"
                      >
                        {item.label}
                      </Link>
                    </li>
                  ))}
                  <li>
                    <a
                      href="https://github.com/Topovium/topovium"
                      className="font-mono text-xs text-mute transition-colors hover:text-chalk"
                    >
                      Source
                    </a>
                  </li>
                </ul>
              </nav>
            </header>

            <main id="main">{children}</main>

            <footer className="mt-24 border-t border-rule px-4 py-10 sm:px-8">
              <div className="flex flex-wrap items-start justify-between gap-8">
                <div>
                  <p className="display text-xl text-chalk">Topovium</p>
                  <p className="mt-2 max-w-sm text-sm text-mute">
                    Free software under the GPL-3.0-or-later. No account, no telemetry by
                    default, no cloud requirement.
                  </p>
                </div>
                <ul className="grid gap-2 font-mono text-xs text-mute">
                  <li>
                    <a className="hover:text-chalk" href="https://github.com/Topovium/topovium">
                      github.com/Topovium/topovium
                    </a>
                  </li>
                  <li>
                    <a
                      className="hover:text-chalk"
                      href="https://github.com/Topovium/topovium/blob/main/AGENTS.md"
                    >
                      Contributor rules
                    </a>
                  </li>
                  <li>
                    <a
                      className="hover:text-chalk"
                      href="https://github.com/Topovium/topovium/blob/main/LICENSE"
                    >
                      GPL-3.0-or-later
                    </a>
                  </li>
                </ul>
              </div>
            </footer>
          </div>
        </div>
      </body>
    </html>
  );
}
