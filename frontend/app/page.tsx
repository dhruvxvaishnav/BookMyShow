'use client';
import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { Film, Star, Clock, Ticket } from 'lucide-react';
import { getShows } from '@/api/shows';
import type { Movie, Show } from '@/types/api';
import EmptyState from '@/components/common/EmptyState';
import { formatPrice } from '@/utils/format';
import { getMoviePosterUrl } from '@/utils/moviePosters';
import styles from './page.module.css';

interface HomeMovie {
  movie: Movie;
  shows: Show[];
  minPrice: number;
}

export default function HomePage() {
  const [shows, setShows] = useState<Show[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');

  const loadShows = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getShows();
      setShows(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load shows.');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { loadShows(); }, [loadShows]);

  const movies = getUniqueMovies(shows);
  const query = search.trim().toLowerCase();
  const filtered = movies.filter(({ movie }) =>
    movie.title.toLowerCase().includes(query) ||
    movie.genre.toLowerCase().includes(query) ||
    movie.language.toLowerCase().includes(query)
  );

  const featured = movies[0] ?? null;

  return (
    <>
      {/* ── Hero ─────────────────────────────────────────── */}
      <section className={styles.hero} aria-label="Featured show">
        {/* Blurred backdrop */}
        <div className={styles.heroBg}>
          {getMoviePosterUrl(featured?.movie) ? (
            <img
              src={getMoviePosterUrl(featured?.movie)!}
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
                  {featured.movie.title}
                </h1>

                <p className={styles.heroSubtitle}>
                  {featured.shows.length} {featured.shows.length === 1 ? 'show' : 'shows'} available
                </p>

                <div className={styles.heroBadges}>
                  <span className={styles.genreBadge}>{featured.movie.genre}</span>
                  <span className={styles.ratingBadge}>
                    <Star size={12} strokeWidth={2} aria-hidden="true" />
                    {featured.movie.rating.toFixed(1)}
                  </span>
                  <span className={styles.timeBadge}>
                    <Clock size={12} strokeWidth={1.5} aria-hidden="true" />
                    {featured.movie.duration_minutes} min
                  </span>
                </div>

                <p className={styles.heroDescription}>
                  {featured.movie.description.slice(0, 160)}
                  {featured.movie.description.length > 160 ? '...' : ''}
                </p>

                <div className={styles.heroActions}>
                  <Link href={`/movies/${featured.movie.movie_id}`} className={styles.bookNowBtn}>
                    <Ticket size={16} strokeWidth={1.5} aria-hidden="true" />
                    Book Now
                  </Link>
                  <Link href="/movies" className={styles.viewAllBtn}>
                    View All Movies
                  </Link>
                </div>
              </div>

              {/* Right column — poster */}
              <div className={styles.heroRight} aria-hidden="true">
                {getMoviePosterUrl(featured.movie) ? (
                  <div className={styles.heroPosterFrame}>
                    <img
                      src={getMoviePosterUrl(featured.movie)!}
                      alt={featured.movie.title}
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
                placeholder="Search movies..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                aria-label="Search movies"
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
              title="No movies available"
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
              {filtered.map((movie, i) => (
                <MovieCard
                  key={movie.movie.movie_id}
                  movie={movie}
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

function getUniqueMovies(
  shows: Show[]
): HomeMovie[] {
  const map = new Map<string, HomeMovie>();

  shows.forEach((show) => {
    if (!show.movie || !show.movie_id) return;

    const existing = map.get(show.movie_id);

    if (!existing) {
      map.set(show.movie_id, {
        movie: show.movie,
        shows: [show],
        minPrice: show.price_per_seat,
      });
      return;
    }

    existing.shows.push(show);
    existing.minPrice = Math.min(existing.minPrice, show.price_per_seat);
  });

  return Array.from(map.values());
}

function MovieCard({
  movie,
  index,
}: {
  movie: HomeMovie;
  index: number;
}) {
  const poster = getMoviePosterUrl(movie.movie);
  const title = movie.movie.title;

  return (
    <article
      className={styles.showCard}
      role="listitem"
      style={{ animationDelay: `${index * 60}ms` }}
    >
      <Link href={`/movies/${movie.movie.movie_id}`} className={styles.showCardLink} tabIndex={-1} aria-hidden="true">
        <div className={styles.showCardPoster}>
          {poster ? (
            <img src={poster} alt={title} className={styles.showCardPosterImg} />
          ) : (
            <div className={styles.showCardPosterPlaceholder}>
              <Film size={32} strokeWidth={0.75} />
            </div>
          )}
        </div>
      </Link>

      <div className={styles.showCardBody}>
        <h3 className={styles.showCardTitle}>{title}</h3>
        <p className={styles.showCardTheatre}>{movie.movie.genre}</p>
        <p className={styles.showCardTime}>
          <Clock size={11} strokeWidth={1.5} aria-hidden="true" />
          {movie.movie.duration_minutes} min
        </p>
        <p className={styles.showCardPrice}>From {formatPrice(movie.minPrice)}</p>

        <Link href={`/movies/${movie.movie.movie_id}`} className={styles.showCardBook} aria-label={`Choose showtime and venue for ${title}`}>
          Book
        </Link>
      </div>
    </article>
  );
}
