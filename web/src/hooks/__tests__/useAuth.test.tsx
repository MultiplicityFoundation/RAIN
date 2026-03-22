import { renderHook, act, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { AuthProvider, useAuth } from '../useAuth';

const authFns = vi.hoisted(() => ({
  getToken: vi.fn(),
  setToken: vi.fn(),
  clearToken: vi.fn(),
  isAuthenticated: vi.fn(),
}));

const apiFns = vi.hoisted(() => ({
  getPublicHealth: vi.fn(),
  pair: vi.fn(),
}));

vi.mock('../../lib/auth', () => authFns);
vi.mock('../../lib/api', () => ({
  getPublicHealth: apiFns.getPublicHealth,
  pair: apiFns.pair,
}));

function wrapper({ children }: { children: ReactNode }) {
  return <AuthProvider>{children}</AuthProvider>;
}

describe('useAuth', () => {
  beforeEach(() => {
    authFns.getToken.mockReset();
    authFns.setToken.mockReset();
    authFns.clearToken.mockReset();
    authFns.isAuthenticated.mockReset();
    authFns.getToken.mockReturnValue(null);
    authFns.isAuthenticated.mockReturnValue(false);

    apiFns.getPublicHealth.mockReset();
    apiFns.pair.mockReset();
    apiFns.getPublicHealth.mockResolvedValue({ require_pairing: true, paired: false });
    apiFns.pair.mockResolvedValue({ token: 'new-token' });

    localStorage.clear();
  });

  it('marks the session authenticated when pairing is not required', async () => {
    apiFns.getPublicHealth.mockResolvedValue({ require_pairing: false, paired: false });

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.requiresPairing).toBe(false);
  });

  it('pairs and persists the returned token', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.pair('654321');
    });

    expect(apiFns.pair).toHaveBeenCalledWith('654321');
    expect(authFns.setToken).toHaveBeenCalledWith('new-token');
    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.token).toBe('new-token');
  });

  it('synchronizes auth state from storage events', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    authFns.getToken.mockReturnValue('synced-token');

    act(() => {
      window.dispatchEvent(new StorageEvent('storage', { key: 'R.A.I.N._token' }));
    });

    expect(result.current.token).toBe('synced-token');
    expect(result.current.isAuthenticated).toBe(true);
  });
});
