import type { ReactNode } from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from '../App';

const authState = vi.hoisted(() => ({
  isAuthenticated: true,
  requiresPairing: false,
  loading: false,
  logout: vi.fn(),
  pair: vi.fn(),
}));

const getAdminPairCodeMock = vi.hoisted(() => vi.fn());

vi.mock('../pages/Dashboard', () => ({
  default: () => <div>dashboard-page</div>,
}));
vi.mock('../pages/AgentChat', () => ({
  default: () => <div>agent-page</div>,
}));
vi.mock('../pages/Tools', () => ({
  default: () => <div>tools-page</div>,
}));
vi.mock('../pages/Cron', () => ({
  default: () => <div>cron-page</div>,
}));
vi.mock('../pages/Integrations', () => ({
  default: () => <div>integrations-page</div>,
}));
vi.mock('../pages/Memory', () => ({
  default: () => <div>memory-page</div>,
}));
vi.mock('../pages/Config', () => ({
  default: () => <div>config-page</div>,
}));
vi.mock('../pages/Cost', () => ({
  default: () => <div>cost-page</div>,
}));
vi.mock('../pages/Logs', () => ({
  default: () => <div>logs-page</div>,
}));
vi.mock('../pages/Doctor', () => ({
  default: () => <div>doctor-page</div>,
}));
vi.mock('../pages/Pairing', () => ({
  default: () => <div>pairing-page</div>,
}));
vi.mock('../components/layout/Layout', () => ({
  default: () => <div><span>layout-shell</span><div data-testid="layout-outlet"><div /></div></div>,
}));
vi.mock('../contexts/ThemeContext', () => ({
  ThemeProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));
vi.mock('../hooks/useDraft', () => ({
  DraftContext: { Provider: ({ children }: { children: ReactNode }) => <>{children}</> },
  useDraftStore: () => ({ draft: null }),
}));
vi.mock('../hooks/useAuth', () => ({
  AuthProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
  useAuth: () => authState,
}));
vi.mock('../lib/api', async () => {
  const actual = await vi.importActual<typeof import('../lib/api')>('../lib/api');
  return {
    ...actual,
    getAdminPairCode: getAdminPairCodeMock,
  };
});

describe('App', () => {
  beforeEach(() => {
    authState.isAuthenticated = true;
    authState.requiresPairing = false;
    authState.loading = false;
    authState.logout.mockReset();
    authState.pair.mockReset();
    getAdminPairCodeMock.mockReset();
    getAdminPairCodeMock.mockResolvedValue({ pairing_code: '123456', pairing_required: true });
  });

  it('shows the loading screen while auth state is resolving', () => {
    authState.loading = true;

    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    expect(screen.getByText('Connecting...')).toBeInTheDocument();
  });

  it('shows pairing UI when the server requires pairing', async () => {
    authState.isAuthenticated = false;
    authState.requiresPairing = true;

    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByText('Your pairing code')).toBeInTheDocument();
    expect(screen.getByText('123456')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Pair' })).toBeDisabled();
  });

  it('logs out when the unauthorized event is dispatched', async () => {
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    );

    window.dispatchEvent(new Event('R.A.I.N.-unauthorized'));

    await waitFor(() => {
      expect(authState.logout).toHaveBeenCalledTimes(1);
    });
  });
});
