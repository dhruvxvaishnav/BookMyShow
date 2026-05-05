import type { Metadata } from 'next';
import './globals.css';
import { ToastProvider } from '@/components/layout/Toast';
import AppShell from '@/components/layout/AppShell';
import { AuthProvider } from '@/contexts/AuthContext';

export const metadata: Metadata = {
  title: 'Cineplex — Royal Cinema Booking',
  description: 'Book your seats at the grandest cinema experience. Dark. Theatrical. Gold.',
  icons: {
    icon: "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>🎬</text></svg>",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <AuthProvider>
          <ToastProvider>
            <AppShell>{children}</AppShell>
          </ToastProvider>
        </AuthProvider>
      </body>
    </html>
  );
}
