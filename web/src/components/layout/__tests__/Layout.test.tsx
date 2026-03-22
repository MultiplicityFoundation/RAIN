import type { ReactNode } from 'react';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import Layout from '../Layout';

vi.mock('../Sidebar', () => ({
  default: () => <aside data-testid="sidebar">sidebar</aside>,
}));

vi.mock('../Header', () => ({
  default: () => <header data-testid="header">header</header>,
}));

vi.mock('../../../App', () => ({
  ErrorBoundary: ({ children }: { children: ReactNode }) => (
    <div data-testid="error-boundary">{children}</div>
  ),
}));

describe('Layout', () => {
  it('renders the shell and nested route content', () => {
    render(
      <MemoryRouter initialEntries={['/logs']}>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/logs" element={<div>logs-content</div>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );

    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('header')).toBeInTheDocument();
    expect(screen.getByTestId('error-boundary')).toHaveTextContent('logs-content');
  });
});
