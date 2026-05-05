'use client';
import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { Film, Star, Clock, Globe } from 'lucide-react';
import { getMovies } from '@/api/movies';
import type { Movie } from '@/types/api';
import { CardSkeleton } from '@/components/common/LoadingSkeleton';
import EmptyState from '@/components/common/EmptyState';
import PageHeader from '@/components/layout/PageHeader';
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

  return (
    <>
      <PageHeader
        title="Movies"
        subtitle="Browse now-showing films and pick your show"
      />
      <div className="container">
        {/* Filters */}
        <div className={styles.filters}>
          <input
            className={styles.search}
            placeholder="Search movies..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            aria-label="Search movies"
          />
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
        </div>

        {error && (
          <div className={styles.errorBanner}>
            <p>{error}</p>
            <button onClick={loadMovies}>Retry</button>
          </div>
        )}

        {isLoading ? (
          <div className={styles.grid}>
            {[...Array(6)].map((_, i) => <CardSkeleton key={i} />)}
          </div>
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No movies found"
            description={search || genreFilter !== 'All' || langFilter !== 'All'
              ? 'Try adjusting your filters.'
              : 'Check back soon for upcoming films.'}
            icon="clapperboard"
            action={
              (search || genreFilter !== 'All' || langFilter !== 'All') ? (
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
          <div className={styles.grid}>
            {filtered.map((movie, i) => (
              <MovieCard key={movie.movie_id} movie={movie} style={{ animationDelay: `${i * 50}ms` }} />
            ))}
          </div>
        )}
      </div>
    </>
  );
}

function MovieCard({ movie, style }: { movie: Movie; style?: React.CSSProperties }) {
  return (
    <div className={styles.card} style={style}>
      <div className={styles.poster}>
        {movie.poster_url ? (
          <img src={movie.poster_url} alt={movie.title} className={styles.posterImg} />
        ) : (
          <div className={styles.posterPlaceholder}>
            <Film size={40} strokeWidth={1} />
          </div>
        )}
        <div className={styles.ratingBadge}>
          <Star size={11} strokeWidth={2} />
          <span>{movie.rating.toFixed(1)}</span>
        </div>
      </div>

      <div className={styles.cardBody}>
        <h3 className={styles.title}>{movie.title}</h3>
        <p className={styles.genre}>{movie.genre}</p>
        <div className={styles.meta}>
          <span className={styles.metaItem}>
            <Clock size={12} strokeWidth={1.5} />
            {movie.duration_minutes} min
          </span>
          <span className={styles.metaItem}>
            <Globe size={12} strokeWidth={1.5} />
            {movie.language}
          </span>
        </div>
        <p className={styles.description}>{movie.description}</p>
        <Link href={`/movies/${movie.movie_id}`} className={styles.viewBtn}>
          Book Tickets
        </Link>
      </div>
    </div>
  );
}
