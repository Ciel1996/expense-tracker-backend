import Axios from 'axios';
import {useAuth} from "./auth.context";

export const AXIOS_INSTANCE = Axios.create({
  baseURL: process.env['NEXT_PUBLIC_API_URL'] || 'http://localhost:3000/api'
});

type CustomClient<T> = (data: {
  url: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  params?: Record<string, any>;
  headers?: Record<string, any>;
  data?: BodyType<unknown>;
  signal?: AbortSignal;
}) => Promise<T>;

export const useCustomClient = <T>(): CustomClient<T> => {
  const token = useAuth();

  return async ({ url, method, params, data, signal }) => {
    const { data: responseData } = await AXIOS_INSTANCE.request<T>({
      url,
      method,
      params,
      signal,
      headers: {
        Authorization: `Bearer ${token}`,
        ...(data?.headers ?? {}),
      },
      // Pass the body directly for POST/PUT requests
      data,
    });
    return responseData;
  };

};

export default useCustomClient;

export type ErrorType<ErrorData> = ErrorData;

export type BodyType<BodyData> = BodyData & { headers?: any };
