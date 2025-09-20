'use client';

import '../app/global.css';
import { PropsWithChildren, useEffect } from 'react';
import { useAuth } from '@./expense-tracker-client';
import { UserDisplay } from './user-display';

/**
 * Centralized application layout used by both App Router and Pages Router.
 * Owns:
 * - global.css import
 * - dark mode initialization
 * - top navigation bar when authenticated
 * - base page container styling
 */
export function AppLayout({ children }: PropsWithChildren) {
  // Initialize dark mode class to match previous App Router behavior
  useEffect(() => {
    if (typeof window === 'undefined') return;
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, []);

  const token = useAuth();

  return (
    <div className="bg-gray-100 dark:bg-black text-gray-900 dark:text-gray-100 min-h-screen">
      {token && (
        <div className="w-full bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
          <div className="flex justify-between items-center">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">Expense Tracker</h1>
            <UserDisplay />
          </div>
        </div>
      )}
      <div className="p-4">{children}</div>
    </div>
  );
}
