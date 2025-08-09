'use client';

import {useAuth, useCurrentUser, useHealthCheck,} from '@./expense-tracker-client';

export default function Index() {
  const token = useAuth();
  const { data: healthStatus} = useHealthCheck();
  // The `useCurrentUser` query will only run when `token` is not null.
  const { data: currentUser } = useCurrentUser({
    query: {
      enabled: !!token,
    },
  });


  if (token) {
    return (
      <h1>
        Welcome to expense-tracker-frontend 👋 <br/>
        <span>Hello {currentUser?.name ?? 'Anonymous'}, </span>
        <span>Message from ExpenseTracker: {healthStatus}</span>
        <br/>
        <button onClick={() => window.location.href = '/api/auth/signout'}>Sign out</button>
      </h1>
    );
  }

  return <button onClick={() => window.location.href = '/api/auth/signin'}>Sign in</button>;
}
