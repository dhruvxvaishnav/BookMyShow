'use client';
import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { Search, Ticket, Clock, Monitor } from 'lucide-react';
import { getShows, getShowAvailability } from '@/api/shows';
import type { Show, ShowAvailability } from '@/types/api';
import Badge from '@/components/common/Badge';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import PageHeader from '@/components/layout/PageHeader';
import { formatDateTime, formatPrice } from '@/utils/format';
import styles from './page.module.css';

export default function HomePage() {
  const [shows, setShows] = useState<Show[]>([]);
  const [availabilities, setAvailabilities] = useState<Record<string, ShowAvailability>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');

  const loadShows = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getShows();
      setShows(data);

      // Load availability for each show in parallel
      const avail = await Promise.all(
        data.map((s) => getShowAvailability(s.show_id).catch(() => null))
      );
      const map: Record<string, ShowAvailability> = {};
      data.forEach((s, i) => {
        if (avail[i]) map[s.show_id] = avail[i]!;
      });
      setAvailabilities(map);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load shows.');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { loadShows(); }, [loadShows]);

  const filtered = shows.filter((s) =>
    s.name.toLowerCase().includes(search.toLowerCase()) ||
    s.theatre_name.toLowerCase().includes(search.toLowerCase())
  );

  function availabilityBadge(showId: string) {
    const a = availabilities[showId];
    if (!a) return null;
    const { available_seats, total_seats, occupancy_percent } = a;
    const pct = occupancy_percent ?? Math.round(((total_seats - available_seats) / total_seats) * 100);
    if (pct >= 80) return <Badge variant="error">{available_seats} left</Badge>;
    if (pct >= 50) return <Badge variant="warning">{available_seats} left</Badge>;
    return <Badge variant="success">{available_seats} seats</Badge>;
  }

  return (
    <>
      <PageHeader
        title="Now Showing"
        subtitle="Select a show to book your seats"
        actions={
          <div className={styles.searchWrap}>
            <Search size={16} strokeWidth={1.5} className={styles.searchIcon} />
            <input
              className={styles.search}
              placeholder="Search shows or theatres..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        }
      />

      <div className="container">
        {error && (
          <div className={styles.errorBanner}>
            <p>{error}</p>
            <button onClick={loadShows} className={styles.retryBtn}>Retry</button>
          </div>
        )}

        {isLoading ? (
          <div className={styles.grid}>
            {[...Array(6)].map((_, i) => <CardSkeleton key={i} />)}
          </div>
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No shows available"
            description={search ? 'Try adjusting your search.' : 'Check back later for upcoming shows.'}
            icon="clapperboard"
            action={
              search ? (
                <button className={styles.clearBtn} onClick={() => setSearch('')}>
                  Clear search
                </button>
              ) : undefined
            }
          />
        ) : (
          <div className={styles.grid}>
            {filtered.map((show, i) => (
              <ShowCard
                key={show.show_id}
                show={show}
                availability={availabilities[show.show_id]}
                badge={availabilityBadge(show.show_id)}
                style={{ animationDelay: `${i * 50}ms` }}
              />
            ))}
          </div>
        )}
      </div>
    </>
  );
}

function ShowCard({
  show,
  availability,
  badge,
  style,
}: {
  show: Show;
  availability?: ShowAvailability;
  badge: React.ReactNode;
  style?: React.CSSProperties;
}) {
  return (
    <div className={styles.card} style={style}>
      <div className={styles.cardScreen}>
        <Monitor size={28} strokeWidth={1} />
        <span>Screen {show.screen_number}</span>
      </div>

      <div className={styles.cardBody}>
        <h3 className={styles.cardTitle}>{show.name}</h3>
        <p className={styles.cardTheatre}>{show.theatre_name}</p>

        <div className={styles.cardMeta}>
          <div className={styles.metaItem}>
            <Clock size={13} strokeWidth={1.5} />
            <span>{formatDateTime(show.start_time)}</span>
          </div>
          <div className={styles.metaItem}>
            <Ticket size={13} strokeWidth={1.5} />
            <span>{formatPrice(show.price_per_seat)} / seat</span>
          </div>
        </div>

        <div className={styles.cardFooter}>
          <div className={styles.availBadge}>{badge}</div>
          <Link href={`/shows/${show.show_id}`} className={styles.bookBtn}>
            Select Seats
          </Link>
        </div>
      </div>
    </div>
  );
}
