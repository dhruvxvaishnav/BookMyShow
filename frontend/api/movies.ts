import client from './client';
import type { Movie, Show } from '@/types/api';

export async function getMovies(): Promise<Movie[]> {
  const res = await client.get<Movie[]>('/movies');
  return res.data;
}

export async function getMovie(movieId: string): Promise<Movie> {
  const res = await client.get<Movie>(`/movies/${movieId}`);
  return res.data;
}

export async function getMovieShows(movieId: string): Promise<Show[]> {
  const res = await client.get<Show[]>(`/movies/${movieId}/shows`);
  return res.data;
}
