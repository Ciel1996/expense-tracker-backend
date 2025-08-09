import React, {ReactNode, useContext, useState} from "react";

type Dispatch = (Auth: string) => void;

const AuthContext = React.createContext<string | null>(null);
const AuthDispatchContext = React.createContext<Dispatch | null>(null);

type AuthProviderProps = {
  children: ReactNode,
  initialState?: string | null,
}

export const AuthProvider =
  ({children, initialState = null}: AuthProviderProps) => {
  const [token, setToken] = useState(initialState);

  return (
    <AuthContext.Provider value={token}>
      <AuthDispatchContext.Provider value={setToken}>
        {children}
      </AuthDispatchContext.Provider>
    </AuthContext.Provider>
  )
};

export const useAuth = (): string | null => {
  return useContext<string | null>(AuthContext);
};

export const useAuthDispatch = (): Dispatch => {
  const context = useContext<Dispatch | null>(AuthDispatchContext);
  if (!context) {
    throw new Error("useAuthDispatch must be used within a AuthProvider");
  }
  return context;
};
