'use client';
import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Plus, Trash2 } from 'lucide-react';
import PageHeader from '@/components/layout/PageHeader';
import Button from '@/components/forms/Button';
import Input from '@/components/forms/Input';
import Select from '@/components/forms/Select';
import { useToast } from '@/components/layout/Toast';
import { createShow } from '@/api/admin';
import { getMovies } from '@/api/movies';
import { getVenues } from '@/api/venues';
import { useRequireAdmin } from '@/hooks/useRequireAuth';
import type { Movie, RowConfig, Venue } from '@/types/api';
import styles from './page.module.css';

const SEAT_TYPE_OPTIONS = [
  { value: 'standard', label: 'Standard (1× price)' },
  { value: 'comfort', label: 'Comfort (1.5× price)' },
  { value: 'recliner', label: 'Recliner (2× price)' },
];

const CREATE_NEW_MOVIE = '__create_new_movie__';

const PRESETS = [
  { label: 'Standard Cinema (4 rows × 10 seats)', rows: [
    { row: 'A', seats: 10, seat_type: 'standard' as const },
    { row: 'B', seats: 10, seat_type: 'standard' as const },
    { row: 'C', seats: 10, seat_type: 'standard' as const },
    { row: 'D', seats: 10, seat_type: 'standard' as const },
  ]},
  { label: 'Comfort Cinema (6 rows × 12 seats)', rows: [
    { row: 'A', seats: 12, seat_type: 'comfort' as const },
    { row: 'B', seats: 12, seat_type: 'comfort' as const },
    { row: 'C', seats: 12, seat_type: 'standard' as const },
    { row: 'D', seats: 12, seat_type: 'standard' as const },
    { row: 'E', seats: 12, seat_type: 'standard' as const },
    { row: 'F', seats: 12, seat_type: 'recliner' as const },
  ]},
];

export default function CreateShowPage() {
  const isAdmin = useRequireAdmin();
  const router = useRouter();
  const toast = useToast();
  const [movies, setMovies] = useState<Movie[]>([]);
  const [venues, setVenues] = useState<Venue[]>([]);
  const [selectedMovieId, setSelectedMovieId] = useState(CREATE_NEW_MOVIE);
  const [selectedVenueId, setSelectedVenueId] = useState('');
  const [name, setName] = useState('');
  const [theatre, setTheatre] = useState('');
  const [screen, setScreen] = useState('1');
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');
  const [price, setPrice] = useState('');
  const [rows, setRows] = useState<RowConfig[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!isAdmin) return;

    Promise.all([getMovies(), getVenues()])
      .then(([movieList, venueList]) => {
        setMovies(movieList);
        setVenues(venueList);
      })
      .catch((err) => {
        toast.showToast(err instanceof Error ? err.message : 'Failed to load movies and venues.', 'error');
      });
  }, [isAdmin, toast]);

  if (!isAdmin) return null;

  const selectedVenue = venues.find((venue) => venue.venue_id === selectedVenueId);
  const movieOptions = [
    { value: CREATE_NEW_MOVIE, label: 'Create movie from show name' },
    ...movies.map((movie) => ({ value: movie.movie_id, label: movie.title })),
  ];
  const venueOptions = [
    { value: '', label: 'Select venue' },
    ...venues.map((venue) => ({ value: venue.venue_id, label: `${venue.name} · ${venue.city}` })),
  ];

  const handleMovieChange = (movieId: string) => {
    setSelectedMovieId(movieId);
    if (movieId === CREATE_NEW_MOVIE) return;

    const movie = movies.find((item) => item.movie_id === movieId);
    if (movie && !name.trim()) setName(movie.title);
  };

  const handleVenueChange = (venueId: string) => {
    setSelectedVenueId(venueId);
    const venue = venues.find((item) => item.venue_id === venueId);
    setTheatre(venue?.name ?? '');
  };

  const addRow = () => {
    const nextLabel = rows.length > 0
      ? String.fromCharCode(rows[rows.length - 1].row.charCodeAt(0) + 1)
      : 'A';
    setRows([...rows, { row: nextLabel, seats: 10, seat_type: 'standard' }]);
  };

  const removeRow = (index: number) => {
    setRows(rows.filter((_, i) => i !== index));
  };

  const updateRow = (index: number, field: keyof RowConfig, value: string | number) => {
    setRows(rows.map((r, i) => i === index ? { ...r, [field]: value } : r));
  };

  const applyPreset = (presetIdx: number) => {
    setRows([...PRESETS[presetIdx].rows]);
  };

  const totalSeats = rows.reduce((sum, r) => sum + r.seats, 0);

  const validate = () => {
    const e: Record<string, string> = {};
    if (!selectedVenueId) e.venue = 'Venue is required.';
    if (!name.trim()) e.name = 'Show name is required.';
    if (!theatre.trim()) e.theatre = 'Theatre name is required.';
    if (!screen || Number(screen) < 1) e.screen = 'Screen number must be at least 1.';
    if (selectedVenue && Number(screen) > selectedVenue.screen_count) {
      e.screen = `Screen number must be between 1 and ${selectedVenue.screen_count}.`;
    }
    if (!startTime) e.startTime = 'Start time is required.';
    if (!endTime) e.endTime = 'End time is required.';
    if (startTime && endTime && new Date(startTime) >= new Date(endTime)) {
      e.endTime = 'End time must be after start time.';
    }
    if (!price || Number(price) <= 0) e.price = 'Price per seat must be greater than 0.';
    if (rows.length === 0) e.rows = 'Add at least one row of seats.';
    rows.forEach((r, i) => {
      if (r.seats < 1 || r.seats > 30) {
        e[`row_${i}_seats`] = 'Seats per row must be between 1 and 30.';
      }
    });
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handleSubmit = async () => {
    if (!validate()) return;
    setIsSubmitting(true);
    try {
      const movieId = selectedMovieId === CREATE_NEW_MOVIE ? undefined : selectedMovieId;
      await createShow({
        show_name: name.trim(),
        theatre_name: theatre.trim(),
        screen_number: Number(screen),
        start_time: Math.floor(new Date(startTime).getTime() / 1000),
        end_time: Math.floor(new Date(endTime).getTime() / 1000),
        price_per_seat: Number(price),
        seat_layout: { rows },
        movie_id: movieId,
        venue_id: selectedVenueId,
      });
      toast.showToast(`Show created with ${totalSeats} seats.`, 'success');
      router.push('/admin');
    } catch (err) {
      toast.showToast(err instanceof Error ? err.message : 'Failed to create show.', 'error');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <>
      <PageHeader
        title="Create Show"
        subtitle="Set up a new show with seat layout"
        backHref="/admin"
      />

      <div className="container">
        <div className={styles.form}>
          {/* Basic info */}
          <div className={styles.section}>
            <h2 className={styles.sectionTitle}>Show Details</h2>
            <div className={styles.fieldGrid}>
              <Select
                label="Movie"
                options={movieOptions}
                value={selectedMovieId}
                onChange={(e) => handleMovieChange(e.target.value)}
                error={errors.movie}
              />
              <Select
                label="Venue"
                options={venueOptions}
                value={selectedVenueId}
                onChange={(e) => handleVenueChange(e.target.value)}
                error={errors.venue}
              />
              <Input
                label="Show Name"
                placeholder="Avengers: Endgame"
                value={name}
                onChange={(e) => setName(e.target.value)}
                error={errors.name}
              />
              <Input
                label="Theatre Name"
                placeholder="PVR Nexus"
                value={theatre}
                onChange={(e) => setTheatre(e.target.value)}
                error={errors.theatre}
              />
              <Input
                label="Screen Number"
                type="number"
                min="1"
                value={screen}
                onChange={(e) => setScreen(e.target.value)}
                error={errors.screen}
              />
              <Input
                label="Start Time"
                type="datetime-local"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                error={errors.startTime}
              />
              <Input
                label="End Time"
                type="datetime-local"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                error={errors.endTime}
              />
              <Input
                label="Price Per Seat (₹)"
                type="number"
                min="1"
                placeholder="250"
                value={price}
                onChange={(e) => setPrice(e.target.value)}
                error={errors.price}
              />
            </div>
          </div>

          {/* Seat layout */}
          <div className={styles.section}>
            <h2 className={styles.sectionTitle}>Seat Layout</h2>

            {/* Presets */}
            <div className={styles.presets}>
              {PRESETS.map((preset, i) => (
                <button
                  key={i}
                  className={styles.presetBtn}
                  onClick={() => applyPreset(i)}
                >
                  {preset.label}
                </button>
              ))}
            </div>

            {errors.rows && <p className={styles.fieldError}>{errors.rows}</p>}

            <div className={styles.rowList}>
              {rows.map((row, i) => (
                <div key={i} className={styles.rowItem}>
                  <input
                    className={styles.rowLabelInput}
                    value={row.row}
                    maxLength={1}
                    onChange={(e) => updateRow(i, 'row', e.target.value.toUpperCase())}
                    placeholder="A"
                    aria-label="Row label"
                  />
                  <span className={styles.rowSep}>×</span>
                  <input
                    className={styles.rowSeatsInput}
                    type="number"
                    min="1"
                    max="30"
                    value={row.seats}
                    onChange={(e) => updateRow(i, 'seats', Number(e.target.value))}
                    aria-label="Number of seats"
                  />
                  <Select
                    options={SEAT_TYPE_OPTIONS}
                    value={row.seat_type}
                    onChange={(e) => updateRow(i, 'seat_type', e.target.value as 'standard' | 'comfort' | 'recliner')}
                  />
                  <button
                    className={styles.removeRowBtn}
                    onClick={() => removeRow(i)}
                    aria-label="Remove row"
                  >
                    <Trash2 size={16} strokeWidth={1.5} />
                  </button>
                </div>
              ))}
            </div>

            <div className={styles.addRowActions}>
              <Button variant="secondary" size="sm" leftIcon={<Plus size={14} strokeWidth={1.5} />} onClick={addRow}>
                Add Row
              </Button>
              {rows.length > 0 && (
                <span className={styles.totalSeats}>{totalSeats} seats total</span>
              )}
            </div>

            {/* Preview */}
            {rows.length > 0 && (
              <div className={styles.preview}>
                <span className={styles.previewLabel}>Preview</span>
                <div className={styles.previewGrid}>
                  {rows.map((row, i) => (
                    <div key={i} className={styles.previewRow}>
                      <span className={styles.previewLabel}>{row.row}</span>
                      {[...Array(row.seats)].map((_, j) => (
                        <div
                          key={j}
                          className={`${styles.previewSeat} ${row.seat_type !== 'standard' ? styles[row.seat_type.toLowerCase()] : ''}`}
                        />
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Submit */}
          <div className={styles.submitRow}>
            <Button variant="secondary" onClick={() => router.push('/admin')}>
              Cancel
            </Button>
            <Button
              variant="primary"
              isLoading={isSubmitting}
              onClick={handleSubmit}
              leftIcon={<Plus size={16} strokeWidth={1.5} />}
            >
              Create Show
            </Button>
          </div>
        </div>
      </div>
    </>
  );
}
