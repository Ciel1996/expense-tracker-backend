'use client';

import {createContext, FC, ReactNode, useContext, useEffect, useState} from "react";
import {useSession} from "next-auth/react";
import {DefaultSession} from "next-auth";

declare module 'next-auth' {
  interface Session extends DefaultSession {
    accessToken?: string;
  }
}

const AuthContext = createContext<string | null>(null);
export const AuthProvider: FC<{ children: ReactNode }> = ({children}) => {
  const {data: session, status} = useSession();
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    if (status === 'authenticated' && session?.accessToken) {
      setToken(session.accessToken);
    } else {
      setToken(null);
    }
  }, [session, status]);


  return (
    <AuthContext.Provider value={token}>
      {children}
    </AuthContext.Provider>
  )
};

export const useAuth = (): string | null => {
  return useContext<string | null>(AuthContext);
};
