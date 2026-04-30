import type { Metadata } from 'next';
import './globals.css';
import { ToastProvider } from '@/components/layout/Toast';
import AppShell from '@/components/layout/AppShell';

export const metadata: Metadata = {
  title: 'BookMyShow — Cinematic Seat Booking',
  description: 'Book your favourite movie seats with a premium cinema experience.',
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
        <ToastProvider>
          <AppShell>{children}</AppShell>
        </ToastProvider>
      </body>
    </html>
  );
}
