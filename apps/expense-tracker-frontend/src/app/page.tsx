'use client';

import {useAuth} from '@./expense-tracker-client';
import {PotsOverview} from "../components/pots-overview";
import {UserDisplay} from "../components/user-display";

export default function Index() {
  const token = useAuth();

  if (token) {
    return (
      <div>
        {/* Top Navigation Bar */}
        <div className="w-full bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
          <div className="flex justify-between items-center">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
              Expense Tracker
            </h1>
            <UserDisplay/>
          </div>
        </div>

        {/* Main Content */}
        <div className="p-4">
          <PotsOverview/>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center">
      <button
        onClick={() => window.location.href = '/api/auth/signin'}
        className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
      >
        Sign in
      </button>
    </div>
  );
}
