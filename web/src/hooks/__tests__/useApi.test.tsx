import { renderHook, act, waitFor } from '@testing-library/react';
import { useDoctor, useMemory } from '../useApi';

const apiFns = vi.hoisted(() => ({
  getStatus: vi.fn(),
  getTools: vi.fn(),
  getCronJobs: vi.fn(),
  getIntegrations: vi.fn(),
  getMemory: vi.fn(),
  getCost: vi.fn(),
  getCliTools: vi.fn(),
  getHealth: vi.fn(),
  runDoctor: vi.fn(),
}));

vi.mock('../../lib/api', () => apiFns);

describe('useApi hooks', () => {
  beforeEach(() => {
    Object.values(apiFns).forEach((fn) => (fn as ReturnType<typeof vi.fn>).mockReset());
    apiFns.getMemory.mockResolvedValue([{ id: 'entry-1', content: 'memory' }]);
    apiFns.runDoctor.mockResolvedValue([{ name: 'disk', status: 'ok' }]);
  });

  it('loads memory data with the requested filters and refetches on demand', async () => {
    const { result } = renderHook(() => useMemory('active', 'notes'));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(apiFns.getMemory).toHaveBeenCalledWith('active', 'notes');
    expect(result.current.data).toEqual([{ id: 'entry-1', content: 'memory' }]);

    await act(async () => {
      result.current.refetch();
    });

    await waitFor(() => {
      expect(apiFns.getMemory).toHaveBeenCalledTimes(2);
    });
  });

  it('normalizes doctor errors and exposes rerun helpers', async () => {
    apiFns.runDoctor.mockRejectedValueOnce('doctor offline');

    const { result } = renderHook(() => useDoctor());

    await act(async () => {
      result.current.run();
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toEqual(new Error('doctor offline'));

    apiFns.runDoctor.mockResolvedValueOnce([{ name: 'network', status: 'ok' }]);

    await act(async () => {
      result.current.refetch();
    });

    await waitFor(() => {
      expect(result.current.data).toEqual([{ name: 'network', status: 'ok' }]);
    });
  });
});
