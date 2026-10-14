import {Providers} from "./providers";
import { AppLayout } from "../components/app-layout";

export const metadata = {
  title: 'Expense.li',
  description: 'Keep track of your household expenses with Expense.li, a simple and intuitive expense tracker app.',
  icons: {
    icon: "/icon.svg",
    apple: "/apple-touch-icon.png",
  },
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
