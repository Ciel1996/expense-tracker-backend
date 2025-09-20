import type { AppProps } from 'next/app';
import { Providers } from '../app/providers';
import { AppLayout } from '../components/app-layout';

export default function MyApp({ Component, pageProps }: AppProps) {
  return (
    <Providers>
      <AppLayout>
        <Component {...pageProps} />
      </AppLayout>
    </Providers>
  );
}
