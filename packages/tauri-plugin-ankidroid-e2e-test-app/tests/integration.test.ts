import { expect, test, describe, beforeAll, afterAll } from 'vitest';

// Mock the Tauri API for testing
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('AnkiDroid Plugin Integration Tests', () => {
  beforeAll(() => {
    // Setup mock responses
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'plugin:ankidroid|list_cards') {
        // Simulate the graceful error handling response with new field names
        return Promise.resolve(
          JSON.stringify([
            {
              id: 1,
              noteId: 1001,
              deckId: 1,
              front: 'AnkiDroid Permission Denied',
              back: "Permission denied when accessing AnkiDroid. Please enable API access in AnkiDroid Settings > Advanced > AnkiDroid API > 'Third party apps'.",
              tags: 'error,permission',
            },
            {
              id: 2,
              noteId: 1002,
              deckId: 1,
              front: 'How to Enable AnkiDroid API',
              back: "1. Open AnkiDroid\\n2. Go to Settings > Advanced\\n3. Find 'AnkiDroid API'\\n4. Enable 'Third party apps'\\n5. Restart this app",
              tags: 'setup,instructions',
            },
          ])
        );
      }
      if (command === 'plugin:ankidroid|create_card') {
        // Simulate successful card creation
        return Promise.resolve(
          JSON.stringify({
            success: true,
            noteId: 12345,
          })
        );
      }
      if (command === 'plugin:ankidroid|get_decks') {
        // Return real deck data
        return Promise.resolve(
          JSON.stringify([
            { id: 1, name: 'Default' },
            { id: 2, name: 'E2E Test Deck' },
          ])
        );
      }
      return Promise.reject(new Error('Unknown command'));
    });
  });

  afterAll(() => {
    vi.clearAllMocks();
  });

  test('should handle AnkiDroid permission errors gracefully', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    const result = await listCards();
    expect(result).toBeDefined();

    // Parse the JSON response
    const cards = JSON.parse(result);
    expect(Array.isArray(cards)).toBe(true);
    expect(cards.length).toBeGreaterThan(0);

    // Check that we get helpful error messages instead of crashes
    const firstCard = cards[0];
    expect(firstCard).toHaveProperty('id');
    expect(firstCard).toHaveProperty('noteId');
    expect(firstCard).toHaveProperty('deckId');
    expect(firstCard).toHaveProperty('front');
    expect(firstCard).toHaveProperty('back');
    expect(firstCard).toHaveProperty('tags');

    // Verify it's a permission error, not a crash
    expect(firstCard.front).toContain('Permission');
    expect(firstCard.back).toContain('Settings');
  });

  test('should return valid JSON structure', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    const result = await listCards();

    // Should be valid JSON
    expect(() => JSON.parse(result)).not.toThrow();

    const cards = JSON.parse(result);

    // Should be an array of card objects
    expect(Array.isArray(cards)).toBe(true);

    // Each card should have required fields
    cards.forEach((card: any) => {
      expect(typeof card.id).toBe('number');
      expect(typeof card.noteId).toBe('number');
      expect(typeof card.deckId).toBe('number');
      expect(typeof card.front).toBe('string');
      expect(typeof card.back).toBe('string');
      expect(typeof card.tags).toBe('string');
      expect(card.front.length).toBeGreaterThan(0);
      expect(card.back.length).toBeGreaterThan(0);
    });
  });

  test('should provide helpful setup instructions', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    const result = await listCards();
    const cards = JSON.parse(result);

    // Should contain setup instructions
    const instructionCard = cards.find(
      (card: any) => card.front.includes('How to Enable') || card.back.includes('Settings')
    );

    expect(instructionCard).toBeDefined();
    expect(instructionCard.back).toContain('AnkiDroid');
    expect(instructionCard.back).toContain('Settings');
  });

  test('should not throw unhandled exceptions', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    // Test that the function doesn't throw, even on errors
    await expect(listCards()).resolves.not.toThrow();

    // Test with network errors
    mockInvoke.mockRejectedValueOnce(new Error('Network error'));
    await expect(listCards()).resolves.not.toThrow();

    // Test with permission errors
    mockInvoke.mockRejectedValueOnce(new Error('SecurityException: Permission denied'));
    await expect(listCards()).resolves.not.toThrow();
  });

  test('should handle various error scenarios gracefully', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    // Test different error scenarios
    const errorScenarios = [
      'SecurityException: Permission denied',
      'ContentProviderNotFoundException',
      'NetworkException: No connection',
      'Unknown error',
    ];

    for (const errorMessage of errorScenarios) {
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage));

      const result = await listCards();
      expect(result).toBeDefined();

      // Should still return valid JSON
      const cards = JSON.parse(result);
      expect(Array.isArray(cards)).toBe(true);
      expect(cards.length).toBeGreaterThan(0);

      // Should contain error information
      const errorCard = cards[0];
      expect(errorCard.tags).toContain('error');
    }
  });
});

describe('Card Creation Tests', () => {
  test('should create card with deck name', async () => {
    const { createCard } = await import('tauri-plugin-ankidroid-js');

    const result = await createCard('Test Front', 'Test Back', 'E2E Test Deck', 'test,e2e');
    expect(result).toBeDefined();

    const response = JSON.parse(result);
    expect(response.success).toBe(true);
    expect(response.noteId).toBeDefined();
    expect(typeof response.noteId).toBe('number');
  });

  test('should handle deck creation if deck does not exist', async () => {
    const { createCard } = await import('tauri-plugin-ankidroid-js');

    const result = await createCard('New Front', 'New Back', 'New Test Deck', 'new');
    expect(result).toBeDefined();

    const response = JSON.parse(result);
    expect(response.success).toBe(true);
  });

  test('should use default deck if none specified', async () => {
    const { createCard } = await import('tauri-plugin-ankidroid-js');

    const result = await createCard('Default Front', 'Default Back');
    expect(result).toBeDefined();

    const response = JSON.parse(result);
    expect(response.success).toBe(true);
  });
});

describe('Deck Management Tests', () => {
  test('should fetch available decks', async () => {
    const { getDecks } = await import('tauri-plugin-ankidroid-js');

    const result = await getDecks();
    expect(result).toBeDefined();

    const decks = JSON.parse(result);
    expect(Array.isArray(decks)).toBe(true);
    expect(decks.length).toBeGreaterThan(0);

    // Check deck structure
    decks.forEach((deck: any) => {
      expect(typeof deck.id).toBe('number');
      expect(typeof deck.name).toBe('string');
      expect(deck.name.length).toBeGreaterThan(0);
    });
  });

  test('should include default deck', async () => {
    const { getDecks } = await import('tauri-plugin-ankidroid-js');

    const result = await getDecks();
    const decks = JSON.parse(result);

    const defaultDeck = decks.find((deck: any) => deck.name === 'Default');
    expect(defaultDeck).toBeDefined();
    expect(defaultDeck.id).toBe(1);
  });
});

describe('Crash Prevention Tests', () => {
  test('should never return undefined or null', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    // Even with severe errors, should return a string
    mockInvoke.mockRejectedValueOnce(new Error('FATAL ERROR'));

    const result = await listCards();
    expect(result).toBeDefined();
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });

  test('should handle malformed responses', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    // Test with malformed responses
    mockInvoke.mockResolvedValueOnce('invalid json{');

    const result = await listCards();
    expect(result).toBeDefined();

    // Should still be valid JSON
    expect(() => JSON.parse(result)).not.toThrow();
  });

  test('should handle timeout scenarios', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js');

    // Simulate timeout
    mockInvoke.mockImplementationOnce(
      () => new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout')), 100))
    );

    await expect(listCards()).resolves.not.toThrow();
  });
});
