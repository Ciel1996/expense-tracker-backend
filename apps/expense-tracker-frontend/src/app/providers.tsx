'use client';

import { SessionProvider } from 'next-auth/react';
import { AuthProvider } from '@./expense-tracker-client';
import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider} from '@tanstack/react-query';

const queryClient = new QueryClient();

export function Providers({ children }: { children: ReactNode }) {
  return (
    <SessionProvider>
      <AuthProvider>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </AuthProvider>
    </SessionProvider>
  );
}
