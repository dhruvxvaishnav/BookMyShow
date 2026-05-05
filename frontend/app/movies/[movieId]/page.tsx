'use client';
import { useState, useEffect, useCallback } from 'react';
import { use } from 'react';
import Link from 'next/link';
import { ArrowLeft, Film, Star, Clock, Globe, MapPin, Monitor, Ticket } from 'lucide-react';
import { getMovie, getMovieShows } from '@/api/movies';
import { getShowAvailability } from '@/api/shows';
import type { Movie, Show, ShowAvailability } from '@/types/api';
import Badge from '@/components/common/Badge';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { formatDateTime, formatPrice } from '@/utils/format';
import styles from './page.module.css';

export default function MovieDetailPage({ params }: { params: Promise<{ movieId: string }> }) {
  const { movieId } = use(params);
  const [movie, setMovie] = useState<Movie | null>(null);
  const [shows, setShows] = useState<Show[]>([]);
  const [availabilities, setAvailabilities] = useState<Record<string, ShowAvailability>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cityFilter, setCityFilter] = useState('All');

  const loadData = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [movieData, showsData] = await Promise.all([
        getMovie(movieId),
        getMovieShows(movieId),
      ]);
      setMovie(movieData);
      setShows(showsData);

      const avail = await Promise.all(
        showsData.map((s) => getShowAvailability(s.show_id).catch(() => null))
      );
      const map: Record<string, ShowAvailability> = {};
      showsData.forEach((s, i) => { if (avail[i]) map[s.show_id] = avail[i]!; });
      setAvailabilities(map);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load movie details.');
    } finally {
      setIsLoading(false);
    }
  }, [movieId]);

  useEffect(() => { loadData(); }, [loadData]);

  const cities = ['All', ...Array.from(new Set(shows.map((s) => s.venue?.city).filter(Boolean) as string[]))];
  const filteredShows = shows.filter((s) =>
    cityFilter === 'All' || s.venue?.city === cityFilter
  );

  if (isLoading) {
    return (
      <div className="container">
        <div className={styles.skeletonHeader} />
        <div className={styles.grid}>
          {[...Array(3)].map((_, i) => <CardSkeleton key={i} />)}
        </div>
      </div>
    );
  }

  if (error || !movie) {
    return (
      <div className="container">
        <EmptyState title="Movie not found" description={error ?? 'This movie could not be loaded.'} icon="clapperboard" />
      </div>
    );
  }

  return (
    <>
      {/* Movie hero */}
      <div className={styles.hero}>
        <div className={styles.heroBg}>
          {movie.poster_url && (
            <img src={movie.poster_url} alt="" className={styles.heroBgImg} aria-hidden="true" />
          )}
          <div className={styles.heroBgOverlay} />
        </div>
        <div className={`container ${styles.heroContent}`}>
          <Link href="/movies" className={styles.backLink}>
            <ArrowLeft size={16} strokeWidth={2} />
            All Movies
          </Link>
          <div className={styles.heroBody}>
            <div className={styles.posterWrap}>
              {movie.poster_url ? (
                <img src={movie.poster_url} alt={movie.title} className={styles.posterImg} />
              ) : (
                <div className={styles.posterPlaceholder}><Film size={48} strokeWidth={1} /></div>
              )}
            </div>
            <div className={styles.heroInfo}>
              <h1 className={styles.movieTitle}>{movie.title}</h1>
              <div className={styles.badges}>
                <span className={styles.genreBadge}>{movie.genre}</span>
                <span className={styles.langBadge}>{movie.language}</span>
              </div>
              <div className={styles.heroMeta}>
                <span className={styles.rating}>
                  <Star size={14} strokeWidth={2} />
                  {movie.rating.toFixed(1)} / 10
                </span>
                <span className={styles.duration}>
                  <Clock size={14} strokeWidth={1.5} />
                  {movie.duration_minutes} min
                </span>
              </div>
              <p className={styles.description}>{movie.description}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Shows section */}
      <div className="container">
        <div className={styles.showsHeader}>
          <h2 className={styles.showsTitle}>Available Shows</h2>
          {cities.length > 1 && (
            <select
              className={styles.citySelect}
              value={cityFilter}
              onChange={(e) => setCityFilter(e.target.value)}
              aria-label="Filter shows by city"
            >
              {cities.map((c) => <option key={c} value={c}>{c}</option>)}
            </select>
          )}
        </div>

        {filteredShows.length === 0 ? (
          <EmptyState
            title="No shows available"
            description="There are no upcoming shows for this movie in the selected city."
            icon="clapperboard"
          />
        ) : (
          <div className={styles.grid}>
            {filteredShows.map((show) => (
              <ShowCard
                key={show.show_id}
                show={show}
                availability={availabilities[show.show_id]}
              />
            ))}
          </div>
        )}
      </div>
    </>
  );
}

function ShowCard({ show, availability }: { show: Show; availability?: ShowAvailability }) {
  const pct = availability
    ? availability.occupancy_percent ?? Math.round(((availability.total_seats - availability.available_seats) / availability.total_seats) * 100)
    : 0;
  const badge = availability
    ? pct >= 80
      ? <Badge variant="error">{availability.available_seats} left</Badge>
      : pct >= 50
        ? <Badge variant="warning">{availability.available_seats} left</Badge>
        : <Badge variant="success">{availability.available_seats} seats</Badge>
    : null;

  return (
    <div className={styles.showCard}>
      <div className={styles.showCardTop}>
        <div className={styles.venueInfo}>
          <p className={styles.venueName}>{show.venue?.name ?? show.theatre_name}</p>
          {show.venue && (
            <p className={styles.venueAddress}>
              <MapPin size={12} strokeWidth={1.5} />
              {show.venue.address}
            </p>
          )}
        </div>
        <div className={styles.screenBadge}>
          <Monitor size={14} strokeWidth={1.5} />
          Screen {show.screen_number}
        </div>
      </div>
      <div className={styles.showCardMid}>
        <span className={styles.showTime}>
          <Clock size={13} strokeWidth={1.5} />
          {formatDateTime(show.start_time)}
        </span>
        <span className={styles.showPrice}>
          <Ticket size={13} strokeWidth={1.5} />
          {formatPrice(show.price_per_seat)} / seat
        </span>
      </div>
      {show.venue?.amenities && show.venue.amenities.length > 0 && (
        <div className={styles.amenities}>
          {show.venue.amenities.slice(0, 3).map((a) => (
            <span key={a} className={styles.amenity}>{a}</span>
          ))}
        </div>
      )}
      <div className={styles.showCardBottom}>
        <div>{badge}</div>
        <Link href={`/shows/${show.show_id}`} className={styles.bookBtn}>
          Select Seats
        </Link>
      </div>
    </div>
  );
}
