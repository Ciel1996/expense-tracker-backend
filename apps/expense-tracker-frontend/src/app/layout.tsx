'use client';

import './global.css';
import { SessionProvider } from "next-auth/react";
import { FC } from 'react';
import { QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {AuthProvider} from "@./expense-tracker-client";

const queryClient = new QueryClient();

const RootLayout: FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <SessionProvider>
      <AuthProvider>
        <QueryClientProvider client={queryClient}>
          <html lang="en">
            <body>{children}</body>
          </html>
        </QueryClientProvider>
      </AuthProvider>
    </SessionProvider>
  );
};

export default RootLayout;
