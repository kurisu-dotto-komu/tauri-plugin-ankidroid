import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import App from '../App';

// Mock the tauri plugin
const mockHello = vi.fn();
const mockListCards = vi.fn();

vi.mock('tauri-plugin-ankidroid-js', () => ({
  hello: mockHello,
  listCards: mockListCards,
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the main heading', () => {
    render(<App />);
    expect(screen.getByText('AnkiDroid Plugin Demo')).toBeInTheDocument();
  });

  it('renders both buttons', () => {
    render(<App />);
    expect(screen.getByText('Test Hello Command')).toBeInTheDocument();
    expect(screen.getByText('List Cards')).toBeInTheDocument();
  });

  it('calls hello command when Test Hello Command button is clicked', async () => {
    mockHello.mockResolvedValue('Hello, World!');

    render(<App />);
    const button = screen.getByText('Test Hello Command');

    fireEvent.click(button);

    await waitFor(() => {
      expect(mockHello).toHaveBeenCalledWith('World');
    });

    expect(screen.getByText('Hello, World!')).toBeInTheDocument();
  });

  it('handles hello command errors gracefully', async () => {
    mockHello.mockRejectedValue(new Error('Test error'));

    render(<App />);
    const button = screen.getByText('Test Hello Command');

    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Error: Error: Test error/)).toBeInTheDocument();
    });
  });

  it('calls listCards command when List Cards button is clicked', async () => {
    const mockCards = JSON.stringify([
      { id: 1, question: 'Test question', answer: 'Test answer', deck: 'Test deck' },
    ]);
    mockListCards.mockResolvedValue(mockCards);

    render(<App />);
    const button = screen.getByText('List Cards');

    fireEvent.click(button);

    await waitFor(() => {
      expect(mockListCards).toHaveBeenCalled();
    });

    expect(screen.getByText('Card Data (JSON):')).toBeInTheDocument();
    expect(screen.getByText(/Test question/)).toBeInTheDocument();
  });

  it('handles listCards command errors gracefully', async () => {
    mockListCards.mockRejectedValue(new Error('List cards error'));

    render(<App />);
    const button = screen.getByText('List Cards');

    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Error: Error: List cards error/)).toBeInTheDocument();
    });
  });

  it('disables buttons during loading', async () => {
    // Mock a slow response to test loading state
    mockHello.mockImplementation(
      () => new Promise((resolve) => setTimeout(() => resolve('Done'), 100))
    );

    render(<App />);
    const button = screen.getByText('Test Hello Command');

    fireEvent.click(button);

    // Button should be disabled and show loading text
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(button).toBeDisabled();

    await waitFor(() => {
      expect(screen.getByText('Done')).toBeInTheDocument();
    });

    // Button should be enabled again
    expect(button).not.toBeDisabled();
  });

  it('handles invalid JSON in listCards response', async () => {
    mockListCards.mockResolvedValue('invalid json{');

    render(<App />);
    const button = screen.getByText('List Cards');

    fireEvent.click(button);

    await waitFor(() => {
      expect(mockListCards).toHaveBeenCalled();
    });

    // Should still display the raw response even if JSON parsing fails
    expect(screen.getByText('Card Data (JSON):')).toBeInTheDocument();
  });
});
