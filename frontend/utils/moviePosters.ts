import type { Movie } from '@/types/api';

const TMDB_W500 = 'https://image.tmdb.org/t/p/w500';

const POSTER_URLS_BY_TITLE: Record<string, string> = {
  Barbie: `${TMDB_W500}/iuFNMS8U5cb6xfzi51Dbkovj7vM.jpg`,
  'KGF: Chapter 2': `${TMDB_W500}/khNVygolU0TxLIDWff5tQlAhZ23.jpg`,
  Pathaan: `${TMDB_W500}/cQOuFy19m0B8kpui7enCthaI8rP.jpg`,
  Jawan: `${TMDB_W500}/uRd0SU6vt84DXobyQ5AI5OG7Mh4.jpg`,
  RRR: `${TMDB_W500}/ljHw5eIMnki3HekwkKwCCHsRSbH.jpg`,
};

export function getMoviePosterUrl(movie?: Pick<Movie, 'title' | 'poster_url'> | null) {
  if (!movie) return null;
  return POSTER_URLS_BY_TITLE[movie.title] ?? movie.poster_url ?? null;
}
