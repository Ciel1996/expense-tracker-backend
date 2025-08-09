import NextAuth from "next-auth";
import KeycloakProvider from "next-auth/providers/keycloak";

export default NextAuth({providers: [
    KeycloakProvider({
      clientId: process.env.EXPENSE_TRACKER_CLIENT_ID as string,
      clientSecret: process.env.EXPENSE_TRACKER_CLIENT_SECRET as string,
      issuer: process.env.EXPENSE_TRACKER_ISSUER as string
    })
  ],
callbacks: {
  async jwt({token, account}) {
    if (account) {
      token.accessToken = account.access_token as string;
    }

    return token;
  },
  async session({session, token}) {
    if (token.accessToken) {
      session.accessToken = token.accessToken as string;
    }
    return session;
  }
}});
