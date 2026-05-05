'use client';
import { useCallback, useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { Clock, Film, Search, Star } from 'lucide-react';
import EmptyState from '@/components/common/EmptyState';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import { getMovies } from '@/api/movies';
import type { Movie } from '@/types/api';
import styles from './page.module.css';

export default function SearchResultsPage() {
  const [movies, setMovies] = useState<Movie[]>([]);
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    setQuery(params.get('q') ?? '');
  }, []);

  const loadMovies = useCallback(async () => {
    setIsLoading(true);
    setError('');
    try {
      setMovies(await getMovies());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load search results.');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { loadMovies(); }, [loadMovies]);

  const results = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return movies;
    return movies.filter((movie) => [
      movie.title,
      movie.genre,
      movie.language,
      movie.description,
    ].some((value) => value.toLowerCase().includes(needle)));
  }, [movies, query]);

  const updateQuery = (value: string) => {
    setQuery(value);
    const next = value.trim() ? `/search?q=${encodeURIComponent(value.trim())}` : '/search';
    window.history.replaceState(null, '', next);
  };

  return (
    <div className={styles.page}>
      <section className={styles.hero}>
        <div className="container">
          <p className="marquee-label">Cineplex Search</p>
          <h1 className={styles.title}>Search Results</h1>
          <div className={styles.searchWrap}>
            <Search size={18} strokeWidth={1.5} className={styles.searchIcon} />
            <input
              className={styles.searchInput}
              value={query}
              onChange={(event) => updateQuery(event.target.value)}
              placeholder="Search by movie, genre, language..."
              autoFocus
            />
          </div>
        </div>
      </section>

      <main className={`container ${styles.resultsArea}`}>
        <div className={styles.resultsHeader}>
          <span className="marquee-label">{query.trim() ? `For "${query.trim()}"` : 'All films'}</span>
          {!isLoading && <span className={styles.count}>{results.length} result{results.length === 1 ? '' : 's'}</span>}
        </div>

        {error && (
          <div className={styles.error} role="alert">
            <span>{error}</span>
            <button onClick={loadMovies}>Retry</button>
          </div>
        )}

        {isLoading ? (
          <div className={styles.grid}>
            {Array.from({ length: 6 }, (_, index) => <CardSkeleton key={index} />)}
          </div>
        ) : results.length === 0 ? (
          <EmptyState
            title="No movies match your search"
            description="Try a different title, genre, or language."
            icon="clapperboard"
            action={<button className={styles.clearBtn} onClick={() => updateQuery('')}>Clear Search</button>}
          />
        ) : (
          <div className={styles.grid} role="list">
            {results.map((movie, index) => (
              <article key={movie.movie_id} className={styles.card} role="listitem" style={{ animationDelay: `${index * 45}ms` }}>
                <div className={styles.poster}>
                  {movie.poster_url ? (
                    <img src={movie.poster_url} alt={movie.title} className={styles.posterImg} />
                  ) : (
                    <div className={styles.posterPlaceholder}>
                      <Film size={40} strokeWidth={0.75} />
                    </div>
                  )}
                  <span className={styles.genreBadge}>{movie.genre}</span>
                  <span className={styles.ratingBadge}>
                    <Star size={10} strokeWidth={2} />
                    {movie.rating.toFixed(1)}
                  </span>
                  <span className={styles.cornerTL} aria-hidden="true" />
                  <span className={styles.cornerBR} aria-hidden="true" />
                </div>
                <div className={styles.cardBody}>
                  <h2 className={styles.movieTitle}>{movie.title}</h2>
                  <p className={styles.description}>{movie.description}</p>
                  <p className={styles.duration}>
                    <Clock size={11} strokeWidth={1.5} />
                    {movie.duration_minutes} min / {movie.language}
                  </p>
                  <Link href={`/movies/${movie.movie_id}`} className={styles.bookBtn}>
                    Book Tickets
                  </Link>
                </div>
              </article>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
