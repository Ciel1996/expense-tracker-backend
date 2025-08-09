'use client';

import {useCurrentUser, useHealthCheck,} from '@./expense-tracker-client';
import {useEffect} from 'react';
import {useAuthDispatch} from "../../../../libs/expense-tracker-client/src/auth.context";

export default function Index() {
  const dispatch = useAuthDispatch();
  const { data: healthStatus, refetch : refetchHealth } = useHealthCheck();
  const { data: currentUser, refetch: refetchUser } = useCurrentUser();

  useEffect(() => {
    dispatch('token');
    setTimeout(() => {
      refetchHealth();
      refetchUser();
    }, 2000);
  }, [refetchHealth, refetchUser, dispatch]);

    return (
        <h1>
          Welcome to expense-tracker-frontend 👋  <br/>
          <span>Hello {currentUser?.name ?? 'Anonymous'}, </span>
          <span>Message from ExpenseTracker: {healthStatus}</span>
        </h1>
    );
}
