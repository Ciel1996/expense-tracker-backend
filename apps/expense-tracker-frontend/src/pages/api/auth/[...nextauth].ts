import NextAuth from "next-auth";
import KeycloakProvider from "next-auth/providers/keycloak";
import {JWT} from "next-auth/jwt";

declare module 'next-auth/jwt' {
  /** Returned by the `jwt` callback and sent to the `session` callback. */
  interface JWT {
    accessToken?: string;
    refreshToken?: string;
    accessTokenExpires?: number;
    error?: string;
  }
}


/**
 * Takes a token, and returns a new token with updated
 * `accessToken` and `accessTokenExpires`. If an error occurs,
 * returns the old token and an error property
 */
async function refreshAccessToken(token: JWT) {
  try {
    const url = `${process.env.EXPENSE_TRACKER_ISSUER}/protocol/openid-connect/token`;

    const response = await fetch(url, {
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      method: 'POST',
      body: new URLSearchParams({
        client_id: process.env.EXPENSE_TRACKER_CLIENT_ID as string,
        client_secret: process.env.EXPENSE_TRACKER_CLIENT_SECRET as string,
        grant_type: 'refresh_token',
        refresh_token: token.refreshToken as string,
      }),
    });

    const refreshedTokens = await response.json();

    if (!response.ok) {
      throw refreshedTokens;
    }

    return {
      ...token,
      accessToken: refreshedTokens.access_token,
      accessTokenExpires: Date.now() + refreshedTokens.expires_in * 1000,
      refreshToken: refreshedTokens.refresh_token ?? token.refreshToken, // Fall back to old refresh token
    };
  } catch (error) {
    console.error('Error refreshing access token', error);
    return {
      ...token,
      error: 'RefreshAccessTokenError',
    };
  }
}

export default NextAuth({
  providers: [
    KeycloakProvider({
      clientId: process.env.EXPENSE_TRACKER_CLIENT_ID as string,
      clientSecret: process.env.EXPENSE_TRACKER_CLIENT_SECRET as string,
      issuer: process.env.EXPENSE_TRACKER_ISSUER as string,
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      if (account) {
        token.accessToken = account.access_token as string;
        token.accessTokenExpires = (account.expires_at as number) * 1000;
        token.refreshToken = account.refresh_token;
      }

      // Return previous token if the access token has not expired yet
      // We check if it's valid with a 1-minute buffer for safety
      if (Date.now() < (token.accessTokenExpires as number) - 60000) {
        return token;
      }

      // Access token has expired, try to update it
      return refreshAccessToken(token);
    },
    async session({ session, token }) {
      if (token.accessToken) {
        session.accessToken = token.accessToken as string;

        // TODO: remove
        console.log(token.accessToken)
      }
      return session;
    },
  },
});
