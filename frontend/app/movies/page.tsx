'use client';
import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { Film, Star, Clock, Search } from 'lucide-react';
import { getMovies } from '@/api/movies';
import type { Movie } from '@/types/api';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import { getMoviePosterUrl } from '@/utils/moviePosters';
import styles from './page.module.css';

const GENRES = ['All', 'Action / Superhero', 'Sci-Fi / Adventure', 'Comedy / Drama', 'Drama', 'Thriller', 'Horror', 'Romance'];
const LANGUAGES = ['All', 'English', 'Hindi', 'Tamil', 'Telugu'];

export default function MoviesPage() {
  const [movies, setMovies] = useState<Movie[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [genreFilter, setGenreFilter] = useState('All');
  const [langFilter, setLangFilter] = useState('All');
  const [search, setSearch] = useState('');

  const loadMovies = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getMovies();
      setMovies(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load movies.');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => { loadMovies(); }, [loadMovies]);

  const filtered = movies.filter((m) => {
    const matchSearch = m.title.toLowerCase().includes(search.toLowerCase());
    const matchGenre = genreFilter === 'All' || m.genre === genreFilter;
    const matchLang = langFilter === 'All' || m.language === langFilter;
    return matchSearch && matchGenre && matchLang;
  });

  const hasActiveFilters = search !== '' || genreFilter !== 'All' || langFilter !== 'All';

  return (
    <>
      {/* ── Sticky Filter Bar ────────────────────────────── */}
      <div className={styles.filterBar} role="search" aria-label="Filter movies">
        <div className="container">
          <div className={styles.filterInner}>
            <div className={styles.searchWrap}>
              <Search size={15} strokeWidth={1.5} className={styles.searchIcon} aria-hidden="true" />
              <input
                className={styles.searchInput}
                placeholder="Search movies..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                aria-label="Search movies"
              />
            </div>

            <select
              className={styles.select}
              value={genreFilter}
              onChange={(e) => setGenreFilter(e.target.value)}
              aria-label="Filter by genre"
            >
              {GENRES.map((g) => <option key={g} value={g}>{g}</option>)}
            </select>

            <select
              className={styles.select}
              value={langFilter}
              onChange={(e) => setLangFilter(e.target.value)}
              aria-label="Filter by language"
            >
              {LANGUAGES.map((l) => <option key={l} value={l}>{l}</option>)}
            </select>

            {hasActiveFilters && (
              <button
                className={styles.clearFiltersBtn}
                onClick={() => { setSearch(''); setGenreFilter('All'); setLangFilter('All'); }}
                aria-label="Clear all filters"
              >
                Clear
              </button>
            )}
          </div>
        </div>
      </div>

      {/* ── Page Content ─────────────────────────────────── */}
      <div className={`container ${styles.pageBody}`}>
        <div className={styles.pageHeadingRow}>
          <h1 className={styles.pageHeading}>
            <span className="marquee-label" style={{ display: 'block', marginBottom: '6px' }}>Cineplex</span>
            Movies
          </h1>
          {!isLoading && !error && (
            <p className={styles.resultCount}>
              {filtered.length} {filtered.length === 1 ? 'film' : 'films'}
            </p>
          )}
        </div>

        {error && (
          <div className={styles.errorBanner} role="alert">
            <p>{error}</p>
            <button onClick={loadMovies} className={styles.retryBtn}>Retry</button>
          </div>
        )}

        {isLoading ? (
          <div className={styles.grid} aria-busy="true">
            {[...Array(6)].map((_, i) => <CardSkeleton key={i} />)}
          </div>
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No movies found"
            description={hasActiveFilters ? 'Try adjusting your filters.' : 'Check back soon for upcoming films.'}
            icon="clapperboard"
            action={
              hasActiveFilters ? (
                <button
                  className={styles.clearBtn}
                  onClick={() => { setSearch(''); setGenreFilter('All'); setLangFilter('All'); }}
                >
                  Clear filters
                </button>
              ) : undefined
            }
          />
        ) : (
          <div className={styles.grid} role="list">
            {filtered.map((movie, i) => (
              <MovieCard key={movie.movie_id} movie={movie} index={i} />
            ))}
          </div>
        )}
      </div>
    </>
  );
}

function MovieCard({ movie, index }: { movie: Movie; index: number }) {
  const poster = getMoviePosterUrl(movie);

  return (
    <article
      className={styles.card}
      role="listitem"
      style={{ animationDelay: `${index * 50}ms` }}
    >
      {/* Poster */}
      <div className={styles.poster}>
        {poster ? (
          <img src={poster} alt={movie.title} className={styles.posterImg} />
        ) : (
          <div className={styles.posterPlaceholder}>
            <Film size={40} strokeWidth={0.75} aria-hidden="true" />
          </div>
        )}

        {/* Genre badge — top left */}
        <span className={styles.genreBadge} aria-label={`Genre: ${movie.genre}`}>
          {movie.genre}
        </span>

        {/* Rating badge — top right */}
        <span className={styles.ratingBadge} aria-label={`Rating: ${movie.rating.toFixed(1)}`}>
          <Star size={10} strokeWidth={2} aria-hidden="true" />
          {movie.rating.toFixed(1)}
        </span>

        {/* Corner bracket hover decoration */}
        <span className={styles.cornerTL} aria-hidden="true" />
        <span className={styles.cornerBR} aria-hidden="true" />
      </div>

      {/* Body */}
      <div className={styles.cardBody}>
        <h3 className={styles.title}>{movie.title}</h3>

        <p className={styles.description}>{movie.description}</p>

        <p className={styles.duration}>
          <Clock size={11} strokeWidth={1.5} aria-hidden="true" />
          {movie.duration_minutes} min &nbsp;&bull;&nbsp; {movie.language}
        </p>

        <Link href={`/movies/${movie.movie_id}`} className={styles.bookBtn} aria-label={`Book tickets for ${movie.title}`}>
          Book Tickets
        </Link>
      </div>
    </article>
  );
}
