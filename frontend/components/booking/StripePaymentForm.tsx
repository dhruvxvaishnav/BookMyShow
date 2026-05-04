'use client';
import { useState } from 'react';
import { useStripe, useElements, PaymentElement } from '@stripe/react-stripe-js';
import Button from '@/components/forms/Button';
import { Lock } from 'lucide-react';
import { formatPrice } from '@/utils/format';
import { useToast } from '@/components/layout/Toast';

interface StripePaymentFormProps {
  amount: number;
  onSuccess: () => void;
}

export default function StripePaymentForm({ amount, onSuccess }: StripePaymentFormProps) {
  const stripe = useStripe();
  const elements = useElements();
  const toast = useToast();
  const [isPaying, setIsPaying] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!stripe || !elements) return;

    setIsPaying(true);

    const { error } = await stripe.confirmPayment({
      elements,
      redirect: 'if_required', // Handle redirect locally to prevent full page reload if possible
    });

    if (error) {
      toast.showToast(error.message || 'Payment failed', 'error');
      setIsPaying(false);
    } else {
      // Payment succeeded
      toast.showToast('Payment successful!', 'success');
      onSuccess();
    }
  };

  return (
    <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <PaymentElement />
      <Button
        variant="primary"
        size="lg"
        isLoading={isPaying}
        disabled={!stripe || isPaying}
        type="submit"
        leftIcon={<Lock size={16} strokeWidth={1.5} />}
        style={{ width: '100%' }}
      >
        Pay {formatPrice(amount)}
      </Button>
    </form>
  );
}
