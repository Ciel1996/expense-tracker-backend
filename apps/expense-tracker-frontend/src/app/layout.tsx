import './global.css';
import {Providers} from "./providers";

export const metadata = {
  title: 'Expense Tracker',
  description: 'Created by Ciel1996',
};

export default function RootLayout({children,}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
    <head>
      <script
        dangerouslySetInnerHTML={{
          __html: `
              (function() {
                if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
                  document.documentElement.classList.add('dark');
                } else {
                  document.documentElement.classList.remove('dark');
                }
              })();
            `,
        }}
      />
    </head>
    <body className="bg-gray-100 dark:bg-black text-gray-900 dark:text-gray-100">
      <Providers>{children}</Providers>
    </body>
    </html>
  );
}
