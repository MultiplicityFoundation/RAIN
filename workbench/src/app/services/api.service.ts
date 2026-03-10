import { Injectable, signal, inject } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { catchError, of, tap } from 'rxjs';

declare const RAIN_API_URL: string;

export interface ComponentHealth {
  status: string;
  updated_at: string;
  last_ok: string | null;
  last_error: string | null;
  restart_count: number;
}

export interface HealthSnapshot {
  pid: number;
  updated_at: string;
  uptime_seconds: number;
  components: Record<string, ComponentHealth>;
}

export interface SystemStatus {
  provider: string;
  model: string;
  temperature: number;
  uptime_seconds: number;
  gateway_port: number;
  locale: string;
  memory_backend: string;
  paired: boolean;
  channels: Record<string, boolean>;
  health: HealthSnapshot;
}

/** Connection state exposed to the UI. */
export type ConnectionState = 'online' | 'offline' | 'connecting';

@Injectable({ providedIn: 'root' })
export class ApiService {
  private http = inject(HttpClient);

  /** Base URL resolved from the build-time define, falling back to localhost. */
  readonly baseUrl: string = (() => {
    try {
      return (RAIN_API_URL || 'http://localhost:3000').replace(/\/$/, '');
    } catch {
      return 'http://localhost:3000';
    }
  })();

  /** Bearer token obtained after pairing (or pre-configured). */
  readonly token = signal<string | null>(null);

  /** Observable connection state. */
  readonly connectionState = signal<ConnectionState>('connecting');

  /** Latest /api/status payload (null while offline or not yet fetched). */
  readonly status = signal<SystemStatus | null>(null);

  /** Latest /api/health payload (null while offline or not yet fetched). */
  readonly health = signal<HealthSnapshot | null>(null);

  // ── Helpers ────────────────────────────────────────────────────

  private authHeaders(): HttpHeaders {
    const t = this.token();
    return t
      ? new HttpHeaders({ Authorization: `Bearer ${t}` })
      : new HttpHeaders();
  }

  // ── Auth / Pairing ─────────────────────────────────────────────

  /**
   * Exchange a one-time pairing code for a bearer token.
   * Stores the token on success.
   */
  pair(code: string) {
    return this.http
      .post<{ token?: string; paired: boolean }>(
        `${this.baseUrl}/pair`,
        null,
        { headers: new HttpHeaders({ 'X-Pairing-Code': code }) }
      )
      .pipe(
        tap(res => {
          if (res.token) {
            this.token.set(res.token);
          }
        })
      );
  }

  // ── Status ────────────────────────────────────────────────────

  /** Fetch /api/status and update local signals. */
  fetchStatus() {
    return this.http
      .get<SystemStatus>(`${this.baseUrl}/api/status`, {
        headers: this.authHeaders(),
      })
      .pipe(
        tap(data => {
          this.status.set(data);
          if (data.health) {
            this.health.set(data.health);
          }
          this.connectionState.set('online');
        }),
        catchError(err => {
          console.warn('[ApiService] /api/status unreachable:', err.message ?? err);
          this.connectionState.set('offline');
          return of(null);
        })
      );
  }

  // ── Health ────────────────────────────────────────────────────

  /** Fetch /api/health directly (independent of /api/status). */
  fetchHealth() {
    return this.http
      .get<HealthSnapshot>(`${this.baseUrl}/api/health`, {
        headers: this.authHeaders(),
      })
      .pipe(
        tap(data => {
          this.health.set(data);
          this.connectionState.set('online');
        }),
        catchError(err => {
          console.warn('[ApiService] /api/health unreachable:', err.message ?? err);
          this.connectionState.set('offline');
          return of(null);
        })
      );
  }
}
