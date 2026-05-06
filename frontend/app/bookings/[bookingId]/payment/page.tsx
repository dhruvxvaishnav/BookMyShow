'use client';
import { useState, useEffect, use } from 'react';
import { useRouter } from 'next/navigation';
import { Lock, X } from 'lucide-react';
import LockTimer from '@/components/booking/LockTimer';
import Button from '@/components/forms/Button';
import Input from '@/components/forms/Input';
import Modal from '@/components/layout/Modal';
import { useToast } from '@/components/layout/Toast';
import { getBooking } from '@/api/bookings';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { initiatePayment, mockGatewayPay } from '@/api/payments';
import { getErrorMessage } from '@/utils/error';
import { formatDateTime, formatPrice, formatSeatList } from '@/utils/format';
import { isFutureCardExpiry, passesLuhn } from '@/utils/validation';
import type { Booking, PaymentInitiateResponse } from '@/types/api';
import { loadStripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';
import StripePaymentForm from '@/components/booking/StripePaymentForm';
import styles from './page.module.css';

const stripePromise = loadStripe(process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY || '');

interface PageProps { params: Promise<{ bookingId: string }> }

export default function PaymentPage({ params }: PageProps) {
  const isAuthed = useRequireAuth();
  const { bookingId } = use(params);
  const router = useRouter();
  const toast = useToast();

  const [booking, setBooking] = useState<Booking | null>(null);
  const [payment, setPayment] = useState<PaymentInitiateResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isPaying, setIsPaying] = useState(false);
  const [isInitLoading, setIsInitLoading] = useState(true);
  const [cardNumber, setCardNumber] = useState('');
  const [cardExpiry, setCardExpiry] = useState('');
  const [cardCvv, setCardCvv] = useState('');
  const [showExpiredModal, setShowExpiredModal] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Load booking
  useEffect(() => {
    getBooking(bookingId)
      .then((b) => {
        setBooking(b);
        // 'payment_pending' is valid here — payment was already initiated
        if (b.status !== 'pending' && b.status !== 'payment_pending') {
          if (b.status === 'success') router.replace(`/bookings/${bookingId}/confirmed`);
          else setShowExpiredModal(true);
        }
      })
      .catch((err) => toast.showToast(getErrorMessage(err), 'error'))
      .finally(() => setIsLoading(false));
  }, [bookingId, router, toast]);

  // Initiate payment only if not already initiated
  useEffect(() => {
    if (!booking || payment) return;
    // Skip if payment was already initiated (page refresh scenario)
    if (booking.status === 'payment_pending') {
      setIsInitLoading(false);
      return;
    }
    if (booking.status !== 'pending') return;
    setIsInitLoading(true);
    initiatePayment(bookingId)
      .then((p) => setPayment(p))
      .catch((err) => {
        toast.showToast(getErrorMessage(err), 'error');
      })
      .finally(() => setIsInitLoading(false));
  }, [booking, bookingId, payment, toast]);

  // Poll booking status every 5 seconds
  useEffect(() => {
    if (!booking) return;
    const interval = setInterval(async () => {
      try {
        const b = await getBooking(bookingId);
        setBooking(b);
        if (b.status === 'expired') {
          setShowExpiredModal(true);
          clearInterval(interval);
        } else if (b.status === 'success') {
          router.replace(`/bookings/${bookingId}/confirmed`);
        }
      } catch { /* silent */ }
    }, 5000);
    return () => clearInterval(interval);
  }, [booking, bookingId, router]);

  if (!isAuthed) return null;

  const validate = () => {
    const e: Record<string, string> = {};
    const rawCard = cardNumber.replace(/\s/g, '');
    if (!passesLuhn(rawCard)) e.cardNumber = 'Enter a valid card number.';
    if (!isFutureCardExpiry(cardExpiry)) e.cardExpiry = 'Enter a valid future expiry as MM/YY.';
    if (!/^\d{3,4}$/.test(cardCvv)) e.cardCvv = 'Enter a valid CVV.';
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handlePay = async () => {
    if (!validate()) return;
    if (!payment) return;
    setIsPaying(true);
    try {
      const cardLast4 = cardNumber.replace(/\s/g, '').slice(-4);
      await mockGatewayPay(payment.payment_intent_id, payment.amount, cardLast4, false);
      toast.showToast('Payment successful!', 'success');
      router.replace(`/bookings/${bookingId}/confirmed`);
    } catch (err) {
      toast.showToast(getErrorMessage(err) + ' Please try again.', 'error');
    } finally {
      setIsPaying(false);
    }
  };

  const formatCardInput = (value: string) => {
    const digits = value.replace(/\D/g, '').slice(0, 16);
    return digits.replace(/(.{4})/g, '$1 ').trim();
  };

  const formatExpiryInput = (value: string) => {
    const digits = value.replace(/\D/g, '').slice(0, 4);
    if (digits.length >= 2) return `${digits.slice(0, 2)}/${digits.slice(2)}`;
    return digits;
  };

  if (isLoading) {
    return (
      <div className="container">
        <div className={styles.loadingCard} />
      </div>
    );
  }

  const seats = booking?.seats ?? [];
  const seatLabels = seats.length > 0
    ? seats.map((seat) => seat.seat_number)
    : booking?.seat_ids ?? [];
  const subtotal = seats.length > 0
    ? seats.reduce((sum, seat) => sum + seat.price, 0)
    : booking?.total_amount ?? payment?.amount ?? 0;
  const total = payment?.amount ?? booking?.total_amount ?? 0;
  const convenienceFee = Math.max(total - subtotal, 0);

  return (
    <>
      <div className="container">
        {booking && (
          <div className={styles.timerBanner}>
            <LockTimer
              expiresAt={booking.expires_at}
              label="Seats release in"
              sublabel="Complete payment before the countdown ends"
            />
          </div>
        )}

        <div className={styles.layout}>
          <section className={styles.summaryCard} aria-label="Order summary">
            <div className={styles.accentBar} />
            <div className={styles.cardBody}>
              <p className={styles.kicker}>Order Summary</p>
              <h1 className={styles.movieTitle}>{booking?.show?.name ?? 'Cinema Ticket'}</h1>

              <div className={styles.showMeta}>
                <span>{booking?.show ? formatDateTime(booking.show.start_time) : 'Show time unavailable'}</span>
                <span>{booking?.show?.theatre_name ?? booking?.show?.venue?.name ?? 'Venue unavailable'}</span>
                {booking?.show && <span>Screen {booking.show.screen_number}</span>}
              </div>

              <div className={styles.seatTable}>
                <div className={styles.tableHeader}>
                  <span>Seat</span>
                  <span>Type</span>
                  <span>Price</span>
                </div>
                {seats.length > 0 ? (
                  seats.map((seat) => (
                    <div key={seat.seat_id} className={styles.tableRow}>
                      <span className={styles.mono}>{seat.seat_number}</span>
                      <span>{seat.seat_type}</span>
                      <span>{formatPrice(seat.price)}</span>
                    </div>
                  ))
                ) : (
                  <div className={styles.tableRow}>
                    <span className={styles.mono}>{formatSeatList(seatLabels)}</span>
                    <span>{seatLabels.length} selected</span>
                    <span>{formatPrice(subtotal)}</span>
                  </div>
                )}
              </div>

              <div className={styles.totals}>
                <div className={styles.summaryRow}>
                  <span>Subtotal</span>
                  <span>{formatPrice(subtotal)}</span>
                </div>
                <div className={styles.summaryRow}>
                  <span>Convenience fee</span>
                  <span>{formatPrice(convenienceFee)}</span>
                </div>
                <div className={styles.totalRow}>
                  <span>Total</span>
                  <span>{formatPrice(total)}</span>
                </div>
              </div>
            </div>
          </section>

          <section className={styles.formCard} aria-label="Payment details">
            <div className={styles.accentBar} />
            <div className={styles.form}>
              <div className={styles.formHeader}>
                <p className={styles.kicker}>Payment</p>
                <h2 className={styles.formTitle}>Complete Payment</h2>
              </div>

              {payment?.client_secret ? (
                <Elements
                  stripe={stripePromise}
	                  options={{
	                    clientSecret: payment.client_secret,
                    appearance: {
                      theme: 'night',
                      variables: {
                        colorPrimary: '#F5A623',
                        colorBackground: '#1E1E23',
                        colorText: '#F5F5F7',
                        colorDanger: '#ef4444',
                        fontFamily: 'Inter, system-ui, sans-serif',
                        spacingUnit: '4px',
                        borderRadius: '4px',
                      },
                    },
                  }}
                >
                  <StripePaymentForm 
                    amount={payment.amount} 
                    onSuccess={() => router.replace(`/bookings/${bookingId}/confirmed`)} 
                  />
                  <Button
                    variant="ghost"
                    onClick={() => router.push(`/bookings/${bookingId}`)}
                    leftIcon={<X size={16} strokeWidth={1.5} />}
                    style={{ marginTop: '1rem' }}
                  >
                    Cancel Payment
                  </Button>
                </Elements>
              ) : (
                <>
                  <div className={styles.fieldGroup}>
                    <Input
                      label="Card Number"
                      placeholder="4242 4242 4242 4242"
                      value={cardNumber}
                      onChange={(e) => setCardNumber(formatCardInput(e.target.value))}
                      error={errors.cardNumber}
                      maxLength={19}
                      inputMode="numeric"
                    />
                  </div>

                  <div className={styles.fieldRow}>
                    <Input
                      label="Expiry"
                      placeholder="MM/YY"
                      value={cardExpiry}
                      onChange={(e) => setCardExpiry(formatExpiryInput(e.target.value))}
                      error={errors.cardExpiry}
                      maxLength={5}
                      inputMode="numeric"
                    />
                    <Input
                      label="CVV"
                      placeholder="•••"
                      type="password"
                      value={cardCvv}
                      onChange={(e) => setCardCvv(e.target.value.replace(/\D/g, '').slice(0, 4))}
                      error={errors.cardCvv}
                      maxLength={4}
                      inputMode="numeric"
                    />
                  </div>

                  <div className={styles.testHint}>
                    Test card: <code>4242 4242 4242 4242</code>
                  </div>

                  <Button
                    variant="primary"
                    size="lg"
                    isLoading={isPaying || isInitLoading}
                    disabled={!payment}
                    onClick={handlePay}
                    leftIcon={<Lock size={16} strokeWidth={1.5} />}
                    style={{ width: '100%' }}
                  >
                    {payment ? `Pay ${formatPrice(payment.amount)}` : 'Preparing payment...'}
                  </Button>

                  <Button
                    variant="ghost"
                    onClick={() => router.push(`/bookings/${bookingId}`)}
                    leftIcon={<X size={16} strokeWidth={1.5} />}
                  >
                    Cancel Payment
                  </Button>
                </>
              )}
            </div>
          </section>
        </div>
      </div>

      <Modal
        isOpen={showExpiredModal}
        onClose={() => { setShowExpiredModal(false); router.replace('/'); }}
        title="Lock Expired"
      >
        <div className={styles.expiredModal}>
          <p>Your lock has expired and the seats have been released.</p>
          <Button variant="primary" onClick={() => router.replace('/')}>
            Browse Shows
          </Button>
        </div>
      </Modal>
    </>
  );
}
