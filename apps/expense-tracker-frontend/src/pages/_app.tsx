import type { AppProps } from 'next/app';
import { Providers } from '../app/providers';
import { AppLayout } from '../components/app-layout';
import Head from 'next/head';

export default function MyApp({ Component, pageProps }: AppProps) {
  return (
    <Providers>
      <Head>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <meta name="color-scheme" content="light dark" />
        <meta name="theme-color" media="(prefers-color-scheme: light)" content="#f3f4f6" />
        <meta name="theme-color" media="(prefers-color-scheme: dark)" content="#000000" />
      </Head>
      <AppLayout>
        <Component {...pageProps} />
      </AppLayout>
    </Providers>
  );
}
