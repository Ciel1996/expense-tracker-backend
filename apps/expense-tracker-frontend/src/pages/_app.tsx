import type { AppProps } from 'next/app';
import { Providers } from '../app/providers';
import { AppLayout } from '../components/app-layout';
import Head from 'next/head';

export default function MyApp({ Component, pageProps }: AppProps) {
  return (
    <Providers>
      <Head>
        <link rel="apple-touch-icon" href="/icon.svg" />
      </Head>
      <AppLayout>
        <Component {...pageProps} />
      </AppLayout>
    </Providers>
  );
}
