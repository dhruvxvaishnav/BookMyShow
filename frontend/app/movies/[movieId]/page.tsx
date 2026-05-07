'use client';
import { useState, useEffect, useCallback, useRef } from 'react';
import { use } from 'react';
import Link from 'next/link';
import { ArrowLeft, Film, Star, Clock, Globe, MapPin, Monitor } from 'lucide-react';
import { getMovie, getMovieShows } from '@/api/movies';
import { getShowAvailability } from '@/api/shows';
import type { Movie, Show, ShowAvailability } from '@/types/api';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { formatDate, formatTime, formatPrice } from '@/utils/format';
import { getMoviePosterUrl } from '@/utils/moviePosters';
import styles from './page.module.css';

export default function MovieDetailPage({ params }: { params: Promise<{ movieId: string }> }) {
  const { movieId } = use(params);
  const [movie, setMovie] = useState<Movie | null>(null);
  const [shows, setShows] = useState<Show[]>([]);
  const [availabilities, setAvailabilities] = useState<Record<string, ShowAvailability>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cityFilter, setCityFilter] = useState('All');
  const [selectedDate, setSelectedDate] = useState<string>('All');
  const showtimesRef = useRef<HTMLDivElement>(null);

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

  // Derive available dates from shows
  const availableDates = ['All', ...Array.from(
    new Set(shows.map((s) => formatDate(s.start_time)))
  )];

  const cities = ['All', ...Array.from(new Set(shows.map((s) => s.venue?.city).filter(Boolean) as string[]))];

  const filteredShows = shows.filter((s) => {
    const matchCity = cityFilter === 'All' || s.venue?.city === cityFilter;
    const matchDate = selectedDate === 'All' || formatDate(s.start_time) === selectedDate;
    return matchCity && matchDate;
  });

  // Group shows by theatre
  const showsByTheatre = filteredShows.reduce<Record<string, Show[]>>((acc, show) => {
    const key = show.venue?.venue_id ?? show.theatre_name;
    if (!acc[key]) acc[key] = [];
    acc[key].push(show);
    return acc;
  }, {});

  function scrollToShowtimes() {
    showtimesRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  if (isLoading) {
    return (
      <>
        <div className={styles.heroSkeleton} aria-busy="true" />
        <div className="container">
          <div className={styles.skeletonGrid}>
            {[...Array(3)].map((_, i) => <CardSkeleton key={i} />)}
          </div>
        </div>
      </>
    );
  }

  if (error || !movie) {
    return (
      <div className="container" style={{ paddingTop: '48px' }}>
        <EmptyState
          title="Movie not found"
          description={error ?? 'This movie could not be loaded.'}
          icon="clapperboard"
        />
      </div>
    );
  }

  const poster = getMoviePosterUrl(movie);

  return (
    <>
      {/* ── Hero ─────────────────────────────────────────── */}
      <section className={styles.hero} aria-label={`${movie.title} details`}>
        {/* Blurred backdrop */}
        <div className={styles.heroBg} aria-hidden="true">
          {poster && (
            <img src={poster} alt="" className={styles.heroBgImg} />
          )}
          <div className={styles.heroBgOverlay} />
        </div>

        <div className={`container ${styles.heroContent}`}>
          {/* Back link */}
          <Link href="/movies" className={styles.backLink}>
            <ArrowLeft size={15} strokeWidth={2} aria-hidden="true" />
            All Movies
          </Link>

          <div className={styles.heroBody}>
            {/* Poster */}
            <div className={styles.posterFrame} aria-hidden="true">
              {poster ? (
                <img src={poster} alt={movie.title} className={styles.posterImg} />
              ) : (
                <div className={styles.posterPlaceholder}>
                  <Film size={48} strokeWidth={0.75} />
                </div>
              )}
            </div>

            {/* Info */}
            <div className={styles.heroInfo}>
              <h1 className={styles.movieTitle}>{movie.title}</h1>

              {/* Badges row */}
              <div className={styles.badgeRow}>
                {/* Rating */}
                <span className={styles.ratingBadge} aria-label={`Rating: ${movie.rating.toFixed(1)} out of 10`}>
                  <Star size={13} strokeWidth={2} aria-hidden="true" />
                  {movie.rating.toFixed(1)}&thinsp;/&thinsp;10
                </span>

                {/* Genre */}
                <span className={styles.genreBadge}>{movie.genre}</span>
              </div>

              {/* Meta line */}
              <div className={styles.metaLine}>
                <span className={styles.metaItem}>
                  <Globe size={13} strokeWidth={1.5} aria-hidden="true" />
                  {movie.language}
                </span>
                <span className={styles.metaDot} aria-hidden="true" />
                <span className={styles.metaItem}>
                  <Clock size={13} strokeWidth={1.5} aria-hidden="true" />
                  {movie.duration_minutes} min
                </span>
              </div>

              {/* Description */}
              <p className={styles.description}>{movie.description}</p>

              {/* CTA */}
              <button
                className={styles.bookTicketsBtn}
                onClick={scrollToShowtimes}
                aria-label="Jump to showtimes section"
              >
                Book Tickets
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* ── Showtimes Section ─────────────────────────────── */}
      <div className="container" ref={showtimesRef}>
        <hr className="ornamental-divider" />

        <div className={styles.showtimesHeader}>
          <span className="marquee-label">Showtimes</span>

          <div className={styles.showtimesFilters}>
            {/* Date pills */}
            {availableDates.length > 1 && (
              <div className={styles.datePills} role="group" aria-label="Filter by date">
                {availableDates.map((d) => (
                  <button
                    key={d}
                    className={selectedDate === d ? `${styles.datePill} ${styles.datePillActive}` : styles.datePill}
                    onClick={() => setSelectedDate(d)}
                    aria-pressed={selectedDate === d}
                  >
                    {d === 'All' ? 'All Dates' : d}
                  </button>
                ))}
              </div>
            )}

            {/* City filter */}
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
        </div>

        {filteredShows.length === 0 ? (
          <EmptyState
            title="No shows available"
            description="There are no upcoming shows for this movie matching your selection."
            icon="clapperboard"
          />
        ) : (
          <div className={styles.theatreList} role="list">
            {Object.entries(showsByTheatre).map(([key, theatreShows]) => {
              const representativeShow = theatreShows[0];
              return (
                <TheatreShowGroup
                  key={key}
                  shows={theatreShows}
                  availabilities={availabilities}
                  representativeShow={representativeShow}
                />
              );
            })}
          </div>
        )}

        {/* Bottom padding */}
        <div style={{ height: 'var(--space-12)' }} />
      </div>
    </>
  );
}

function TheatreShowGroup({
  shows,
  availabilities,
  representativeShow,
}: {
  shows: Show[];
  availabilities: Record<string, ShowAvailability>;
  representativeShow: Show;
}) {
  return (
    <div className={styles.theatreGroup} role="listitem">
      {/* Theatre header */}
      <div className={styles.theatreHeader}>
        <div className={styles.theatreInfo}>
          <p className={styles.theatreName}>
            {representativeShow.venue?.name ?? representativeShow.theatre_name}
          </p>
          {representativeShow.venue?.address && (
            <p className={styles.theatreAddress}>
              <MapPin size={11} strokeWidth={1.5} aria-hidden="true" />
              {representativeShow.venue.address}
            </p>
          )}
        </div>

        {/* Amenity badges */}
        {representativeShow.venue?.amenities && representativeShow.venue.amenities.length > 0 && (
          <div className={styles.amenities} aria-label="Theatre amenities">
            {representativeShow.venue.amenities.slice(0, 4).map((a) => (
              <span key={a} className={styles.amenityBadge}>{a}</span>
            ))}
          </div>
        )}
      </div>

      {/* Showtime pills */}
      <div className={styles.showtimePills} role="list" aria-label="Available showtimes">
        {shows.map((show) => {
          const avail = availabilities[show.show_id];
          const isSoldOut = avail
            ? avail.available_seats === 0
            : false;
          const isAlmostFull = avail
            ? (avail.occupancy_percent ?? 0) >= 80 && avail.available_seats > 0
            : false;

          return (
            <ShowtimePill
              key={show.show_id}
              show={show}
              isSoldOut={isSoldOut}
              isAlmostFull={isAlmostFull}
              availability={avail}
            />
          );
        })}
      </div>
    </div>
  );
}

function ShowtimePill({
  show,
  isSoldOut,
  isAlmostFull,
  availability,
}: {
  show: Show;
  isSoldOut: boolean;
  isAlmostFull: boolean;
  availability?: ShowAvailability;
}) {
  const pillClass = [
    styles.showtimePill,
    isSoldOut ? styles.showtimePillSoldOut : '',
    isAlmostFull ? styles.showtimePillAlmostFull : '',
    !isSoldOut && !isAlmostFull ? styles.showtimePillAvailable : '',
  ].filter(Boolean).join(' ');

  const content = (
    <>
      <span className={styles.showtimeTime}>{formatTime(show.start_time)}</span>
      <span className={styles.showtimePrice}>{formatPrice(show.price_per_seat)}</span>
      {availability && !isSoldOut && (
        <span className={styles.showtimeSeats}>
          {availability.available_seats} left
        </span>
      )}
      {isSoldOut && (
        <span className={styles.soldOutLabel}>Sold Out</span>
      )}
    </>
  );

  if (isSoldOut) {
    return (
      <div className={pillClass} role="listitem" aria-label={`${formatTime(show.start_time)} - Sold Out`} aria-disabled="true">
        {content}
      </div>
    );
  }

  return (
    <Link
      href={`/shows/${show.show_id}`}
      className={pillClass}
      role="listitem"
      aria-label={`Book ${formatTime(show.start_time)} show at ${formatPrice(show.price_per_seat)} per seat`}
    >
      {content}
    </Link>
  );
}
