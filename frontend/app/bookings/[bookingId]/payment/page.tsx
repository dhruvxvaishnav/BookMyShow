'use client';
import { useState, useEffect, useCallback, use } from 'react';
import { useRouter } from 'next/navigation';
import { Lock } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import LockTimer from '@/components/booking/LockTimer';
import Button from '@/components/forms/Button';
import Input from '@/components/forms/Input';
import Modal from '@/components/layout/Modal';
import { useToast } from '@/components/layout/Toast';
import { getBooking } from '@/api/bookings';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { initiatePayment, mockGatewayPay } from '@/api/payments';
import { getErrorMessage } from '@/utils/error';
import { formatPrice } from '@/utils/format';
import type { Booking, PaymentInitiateResponse } from '@/types/api';
import styles from './page.module.css';

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
        if (b.status !== 'Pending') {
          if (b.status === 'Success') router.replace(`/bookings/${bookingId}/confirmed`);
          else setShowExpiredModal(true);
        }
      })
      .catch((err) => toast.showToast(getErrorMessage(err), 'error'))
      .finally(() => setIsLoading(false));
  }, [bookingId, router, toast]);

  // Initiate payment on load
  useEffect(() => {
    if (!booking || payment) return;
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
        if (b.status === 'Expired') {
          setShowExpiredModal(true);
          clearInterval(interval);
        } else if (b.status === 'Success') {
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
    if (!/^\d{16}$/.test(rawCard)) e.cardNumber = 'Enter a valid 16-digit card number.';
    if (!/^\d{2}\/\d{2}$/.test(cardExpiry)) e.cardExpiry = 'Enter expiry as MM/YY.';
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
      <>
        <PageHeader title="Payment" backHref={`/bookings/${bookingId}`} />
        <div className="container"><div className={styles.loadingCard} /></div>
      </>
    );
  }

  return (
    <>
      <PageHeader
        title="Complete Payment"
        backHref={`/bookings/${bookingId}`}
      />

      <div className="container">
        <div className={styles.layout}>
          {/* Left: form */}
          <div className={styles.formCard}>
            {booking && (
              <LockTimer
                expiresAt={booking.expires_at}
                label="Complete payment before"
              />
            )}

            <div className={styles.form}>
              <h3 className={styles.formTitle}>Order Summary</h3>
              <div className={styles.summary}>
                <div className={styles.summaryRow}>
                  <span>Seats</span>
                  <span>{booking?.seat_ids.join(', ')}</span>
                </div>
                <div className={styles.summaryRow}>
                  <span>Amount</span>
                  <span className={styles.amount}>{payment ? formatPrice(payment.amount) : '—'}</span>
                </div>
              </div>

              <div className={styles.divider} />

              <h3 className={styles.formTitle}>Payment Details</h3>

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
              >
                Cancel Payment
              </Button>
            </div>
          </div>

          {/* Right: visual */}
          <div className={styles.sideCard}>
            <div className={styles.cardVisual}>
              <div className={styles.cardChip} />
              <div className={styles.cardNum}>
                {cardNumber || '•••• •••• •••• ••••'}
              </div>
              <div className={styles.cardBottom}>
                <div>
                  <div className={styles.cardLabel}>VALID THRU</div>
                  <div className={styles.cardExpiry}>{cardExpiry || 'MM/YY'}</div>
                </div>
                <div className={styles.cardBrand}>VISA</div>
              </div>
            </div>
            <p className={styles.secureNote}>
              Your payment is secured with 256-bit encryption.
            </p>
          </div>
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