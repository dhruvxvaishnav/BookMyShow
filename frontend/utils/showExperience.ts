import type { Show } from '@/types/api';

export type ShowExperience = 'all' | 'normal' | 'luxe' | 'imax';

export const SHOW_EXPERIENCE_LABELS: Record<ShowExperience, string> = {
  all: 'All Formats',
  normal: 'Normal',
  luxe: 'LUXE',
  imax: 'IMAX',
};

export function getShowExperience(show: Show): Exclude<ShowExperience, 'all'> {
  const venueName = show.venue?.name.toLowerCase() ?? '';
  const theatreName = show.theatre_name.toLowerCase();
  const amenities = show.venue?.amenities.map((amenity) => amenity.toLowerCase()) ?? [];

  if (venueName.includes('imax') || theatreName.includes('imax') || amenities.some((amenity) => amenity.includes('imax'))) {
    return 'imax';
  }

  if (venueName.includes('gold class') || theatreName.includes('gold class') || theatreName.includes('luxe')) {
    return 'luxe';
  }

  return 'normal';
}
