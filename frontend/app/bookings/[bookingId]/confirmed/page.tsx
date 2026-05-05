'use client';
import { useState, useEffect, use } from 'react';
import { useRouter } from 'next/navigation';
import { CheckCircle, Printer } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import TicketDisplay from '@/components/booking/TicketDisplay';
import Button from '@/components/forms/Button';
import { BookingSkeleton } from '@/components/common/LoadingSkeleton';
import { useToast } from '@/components/layout/Toast';
import { getBooking } from '@/api/bookings';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { getErrorMessage } from '@/utils/error';
import type { Booking } from '@/types/api';
import styles from './page.module.css';

interface PageProps { params: Promise<{ bookingId: string }> }

export default function ConfirmedPage({ params }: PageProps) {
  const isAuthed = useRequireAuth();
  const { bookingId } = use(params);
  const router = useRouter();
  const toast = useToast();
  const [booking, setBooking] = useState<Booking | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [confetti, setConfetti] = useState(false);

  useEffect(() => {
    getBooking(bookingId)
      .then((b) => {
        setBooking(b);
        if (b.status !== 'Success') {
          router.replace(`/bookings/${bookingId}`);
        } else {
          // Fire confetti
          setConfetti(true);
          setTimeout(() => setConfetti(false), 3000);
        }
      })
      .catch((err) => setError(getErrorMessage(err)))
      .finally(() => setIsLoading(false));
  }, [bookingId, router]);

  if (!isAuthed) return null;

  if (isLoading) {
    return (
      <>
        <PageHeader title="Booking Confirmed" />
        <div className="container"><BookingSkeleton /></div>
      </>
    );
  }

  if (error || !booking) {
    return (
      <>
        <PageHeader title="Booking Confirmed" backHref="/" />
        <div className="container">
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '48px 0' }}>
            {error ?? 'Booking not found.'}
          </p>
        </div>
      </>
    );
  }

  return (
    <>
      <PageHeader title="Booking Confirmed!" backHref="/" />

      {/* Confetti */}
      {confetti && <Confetti />}

      <div className="container">
        <div className={styles.wrapper}>
          <div className={styles.successBar}>
            <CheckCircle size={24} strokeWidth={1.5} />
            <span>Your booking is confirmed!</span>
          </div>

          <div className={styles.ticketWrap}>
            <TicketDisplay booking={booking} />
          </div>

          <div className={styles.actions} style={{ flexWrap: 'wrap' }}>
            <Button variant="secondary" onClick={() => window.print()} leftIcon={<Printer size={16} />}>
              Download PDF / Print
            </Button>
            <Button variant="secondary" onClick={() => router.push('/')}>
              Browse More Shows
            </Button>
            <Button variant="primary" onClick={() => router.push('/my-bookings')}>
              View My Bookings
            </Button>
          </div>
        </div>
      </div>
    </>
  );
}

function Confetti() {
  const pieces = Array.from({ length: 60 }, (_, i) => ({
    id: i,
    left: `${Math.random() * 100}%`,
    delay: `${Math.random() * 2}s`,
    color: ['#F5A623', '#D4A843', '#7B1F1F', '#EF4444'][Math.floor(Math.random() * 4)],
    size: 6 + Math.random() * 8,
  }));

  return (
    <div className={styles.confettiContainer} aria-hidden="true">
      {pieces.map((p) => (
        <div
          key={p.id}
          className={styles.confettiPiece}
          style={{
            left: p.left,
            animationDelay: p.delay,
            backgroundColor: p.color,
            width: p.size,
            height: p.size,
            borderRadius: Math.random() > 0.5 ? '50%' : '2px',
          }}
        />
      ))}
    </div>
  );
}
