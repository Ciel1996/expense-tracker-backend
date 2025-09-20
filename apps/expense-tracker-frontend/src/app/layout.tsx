import {Providers} from "./providers";
import { AppLayout } from "../components/app-layout";

export const metadata = {
  title: 'Expense Tracker',
  description: 'Created by Ciel1996',
};

export default function RootLayout({children,}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <Providers>
          <AppLayout>{children}</AppLayout>
        </Providers>
      </body>
    </html>
  );
}
