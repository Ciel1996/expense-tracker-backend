'use client';

import {useAuth} from '@./expense-tracker-client';
import {PotsOverview} from "../components/pots-overview";

export default function Index() {
  const token = useAuth();

  if (token) {
    // AppLayout handles the header and padding; just render the main content.
    return <PotsOverview/>;
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
