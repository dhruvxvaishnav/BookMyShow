'use client';
import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { Film, Star, Clock, Ticket } from 'lucide-react';
import { getShows, getShowAvailability } from '@/api/shows';
import type { Show, ShowAvailability } from '@/types/api';
import Badge from '@/components/common/Badge';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { formatTime, formatPrice } from '@/utils/format';
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

  const featured = shows[0] ?? null;

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
      {/* ── Hero ─────────────────────────────────────────── */}
      <section className={styles.hero} aria-label="Featured show">
        {/* Blurred backdrop */}
        <div className={styles.heroBg}>
          {featured?.movie?.poster_url ? (
            <img
              src={featured.movie.poster_url}
              alt=""
              className={styles.heroBgImg}
              aria-hidden="true"
            />
          ) : (
            <div className={styles.heroBgGradient} aria-hidden="true" />
          )}
          <div className={styles.heroBgOverlay} aria-hidden="true" />
        </div>

        <div className={`container ${styles.heroContent}`}>
          {isLoading ? (
            <div className={styles.heroSkeleton} aria-busy="true" />
          ) : featured ? (
            <div className={styles.heroInner}>
              {/* Left column */}
              <div className={styles.heroLeft}>
                <p className="marquee-label" style={{ marginBottom: '16px' }}>Now Featuring</p>

                <h1 className={styles.heroTitle}>
                  {featured.movie?.title ?? featured.name}
                </h1>

                <p className={styles.heroSubtitle}>
                  {featured.venue?.name ?? featured.theatre_name}
                </p>

                <div className={styles.heroBadges}>
                  {featured.movie?.genre && (
                    <span className={styles.genreBadge}>{featured.movie.genre}</span>
                  )}
                  {featured.movie?.rating != null && (
                    <span className={styles.ratingBadge}>
                      <Star size={12} strokeWidth={2} aria-hidden="true" />
                      {featured.movie.rating.toFixed(1)}
                    </span>
                  )}
                  {featured.start_time && (
                    <span className={styles.timeBadge}>
                      <Clock size={12} strokeWidth={1.5} aria-hidden="true" />
                      {formatTime(featured.start_time)}
                    </span>
                  )}
                </div>

                {featured.movie?.description && (
                  <p className={styles.heroDescription}>
                    {featured.movie.description.slice(0, 160)}
                    {featured.movie.description.length > 160 ? '…' : ''}
                  </p>
                )}

                <div className={styles.heroActions}>
                  <Link href={`/shows/${featured.show_id}`} className={styles.bookNowBtn}>
                    <Ticket size={16} strokeWidth={1.5} aria-hidden="true" />
                    Book Now
                  </Link>
                  <Link href="/movies" className={styles.viewAllBtn}>
                    View All Shows
                  </Link>
                </div>
              </div>

              {/* Right column — poster */}
              <div className={styles.heroRight} aria-hidden="true">
                {featured.movie?.poster_url ? (
                  <div className={styles.heroPosterFrame}>
                    <img
                      src={featured.movie.poster_url}
                      alt={featured.movie?.title ?? featured.name}
                      className={styles.heroPosterImg}
                    />
                  </div>
                ) : (
                  <div className={styles.filmStripPlaceholder}>
                    <Film size={64} strokeWidth={0.75} />
                  </div>
                )}
              </div>
            </div>
          ) : error ? null : (
            <div className={styles.heroEmpty}>
              <Film size={48} strokeWidth={0.75} />
              <p>No featured shows at the moment.</p>
            </div>
          )}
        </div>
      </section>

      <hr className="ornamental-divider" style={{ margin: '0 24px' }} />

      {/* ── Now Showing ──────────────────────────────────── */}
      <section className={styles.nowShowingSection} aria-label="Now Showing">
        <div className="container">
          <div className={styles.sectionHeader}>
            <span className="marquee-label">Now Showing</span>
            <div className={styles.searchWrap}>
              <input
                className={styles.search}
                placeholder="Search shows or theatres..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                aria-label="Search shows or theatres"
              />
            </div>
          </div>

          {error && (
            <div className={styles.errorBanner} role="alert">
              <p>{error}</p>
              <button onClick={loadShows} className={styles.retryBtn}>Retry</button>
            </div>
          )}

          {isLoading ? (
            <div className={styles.scrollRow} aria-busy="true">
              {[...Array(6)].map((_, i) => (
                <div key={i} className={styles.cardSkeleton} />
              ))}
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
            <div className={styles.scrollRow} role="list">
              {filtered.map((show, i) => (
                <ShowCard
                  key={show.show_id}
                  show={show}
                  availability={availabilities[show.show_id]}
                  badge={availabilityBadge(show.show_id)}
                  index={i}
                />
              ))}
            </div>
          )}
        </div>
      </section>
    </>
  );
}

function ShowCard({
  show,
  availability,
  badge,
  index,
}: {
  show: Show;
  availability?: ShowAvailability;
  badge: React.ReactNode;
  index: number;
}) {
  const poster = show.movie?.poster_url ?? null;
  const title = show.movie?.title ?? show.name;

  return (
    <article
      className={styles.showCard}
      role="listitem"
      style={{ animationDelay: `${index * 60}ms` }}
    >
      <Link href={`/shows/${show.show_id}`} className={styles.showCardLink} tabIndex={-1} aria-hidden="true">
        <div className={styles.showCardPoster}>
          {poster ? (
            <img src={poster} alt={title} className={styles.showCardPosterImg} />
          ) : (
            <div className={styles.showCardPosterPlaceholder}>
              <Film size={32} strokeWidth={0.75} />
            </div>
          )}
          {badge && <div className={styles.availOverlay}>{badge}</div>}
        </div>
      </Link>

      <div className={styles.showCardBody}>
        <h3 className={styles.showCardTitle}>{title}</h3>
        <p className={styles.showCardTheatre}>{show.venue?.name ?? show.theatre_name}</p>
        {show.start_time && (
          <p className={styles.showCardTime}>
            <Clock size={11} strokeWidth={1.5} aria-hidden="true" />
            {formatTime(show.start_time)}
          </p>
        )}
        <p className={styles.showCardPrice}>{formatPrice(show.price_per_seat)}</p>

        <Link href={`/shows/${show.show_id}`} className={styles.showCardBook} aria-label={`Book ${title}`}>
          Book
        </Link>
      </div>
    </article>
  );
}
