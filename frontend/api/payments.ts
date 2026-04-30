import apiClient from './client';
import type { PaymentInitiateResponse, MockGatewayPayRequest, Payment } from '@/types/api';

export async function initiatePayment(bookingId: string): Promise<PaymentInitiateResponse> {
  const response = await apiClient.post<PaymentInitiateResponse>(
    `/bookings/${bookingId}/payment/initiate`
  );
  return response.data;
}

export async function getPayment(paymentId: string): Promise<Payment> {
  const response = await apiClient.get<Payment>(`/payments/${paymentId}`);
  return response.data;
}

export async function mockGatewayPay(
  paymentIntentId: string,
  amount: number,
  cardLast4: string,
  simulateFailure = false
): Promise<void> {
  const payload: MockGatewayPayRequest = {
    payment_intent_id: paymentIntentId,
    amount,
    card_last4: cardLast4,
    simulate_failure: simulateFailure,
  };
  await apiClient.post('/mock-gateway/pay', payload);
}
