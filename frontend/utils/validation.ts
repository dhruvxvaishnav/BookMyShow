export function isValidEmail(value: string): boolean {
  const email = value.trim();
  return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(email) && email.length <= 254;
}

export function getPasswordStrengthError(value: string): string | null {
  if (value.length < 8) return 'Password must be at least 8 characters';
  if (!/[A-Z]/.test(value)) return 'Password must include an uppercase letter';
  if (!/[a-z]/.test(value)) return 'Password must include a lowercase letter';
  if (!/\d/.test(value)) return 'Password must include a number';
  return null;
}

export function passesLuhn(value: string): boolean {
  const digits = value.replace(/\D/g, '');
  if (digits.length < 12 || digits.length > 19) return false;

  let sum = 0;
  let shouldDouble = false;
  for (let i = digits.length - 1; i >= 0; i -= 1) {
    let digit = Number(digits[i]);
    if (shouldDouble) {
      digit *= 2;
      if (digit > 9) digit -= 9;
    }
    sum += digit;
    shouldDouble = !shouldDouble;
  }
  return sum % 10 === 0;
}

export function isFutureCardExpiry(value: string): boolean {
  if (!/^\d{2}\/\d{2}$/.test(value)) return false;
  const [monthRaw, yearRaw] = value.split('/');
  const month = Number(monthRaw);
  if (month < 1 || month > 12) return false;

  const now = new Date();
  const year = 2000 + Number(yearRaw);
  const expiryEnd = new Date(year, month, 0, 23, 59, 59, 999);
  return expiryEnd >= now;
}
