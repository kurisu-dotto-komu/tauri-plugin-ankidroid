import { expect, test, describe, beforeAll, afterAll } from 'vitest'

// Mock the Tauri API for testing
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

describe('AnkiDroid Plugin Integration Tests', () => {
  beforeAll(() => {
    // Setup mock responses
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'plugin:ankidroid|list_cards') {
        // Simulate the graceful error handling response
        return Promise.resolve(JSON.stringify([
          {
            "id": 1,
            "question": "AnkiDroid Permission Denied",
            "answer": "Permission denied when accessing AnkiDroid. Please enable API access in AnkiDroid Settings > Advanced > AnkiDroid API > 'Third party apps'.",
            "deck": "Permission Error",
            "note": "SecurityException - AnkiDroid API access is disabled"
          },
          {
            "id": 2,
            "question": "How to Enable AnkiDroid API", 
            "answer": "1. Open AnkiDroid\\n2. Go to Settings > Advanced\\n3. Find 'AnkiDroid API'\\n4. Enable 'Third party apps'\\n5. Restart this app",
            "deck": "Setup Instructions",
            "note": "Step-by-step guide to enable external access"
          }
        ]))
      }
      return Promise.reject(new Error('Unknown command'))
    })
  })

  afterAll(() => {
    vi.clearAllMocks()
  })

  test('should handle AnkiDroid permission errors gracefully', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    const result = await listCards()
    expect(result).toBeDefined()
    
    // Parse the JSON response
    const cards = JSON.parse(result)
    expect(Array.isArray(cards)).toBe(true)
    expect(cards.length).toBeGreaterThan(0)
    
    // Check that we get helpful error messages instead of crashes
    const firstCard = cards[0]
    expect(firstCard).toHaveProperty('id')
    expect(firstCard).toHaveProperty('question')
    expect(firstCard).toHaveProperty('answer')
    expect(firstCard).toHaveProperty('deck')
    expect(firstCard).toHaveProperty('note')
    
    // Verify it's a permission error, not a crash
    expect(firstCard.question).toContain('Permission')
    expect(firstCard.answer).toContain('Settings')
  })

  test('should return valid JSON structure', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    const result = await listCards()
    
    // Should be valid JSON
    expect(() => JSON.parse(result)).not.toThrow()
    
    const cards = JSON.parse(result)
    
    // Should be an array of card objects
    expect(Array.isArray(cards)).toBe(true)
    
    // Each card should have required fields
    cards.forEach((card: any) => {
      expect(typeof card.id).toBe('number')
      expect(typeof card.question).toBe('string')
      expect(typeof card.answer).toBe('string')
      expect(typeof card.deck).toBe('string')
      expect(card.question.length).toBeGreaterThan(0)
      expect(card.answer.length).toBeGreaterThan(0)
    })
  })

  test('should provide helpful setup instructions', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    const result = await listCards()
    const cards = JSON.parse(result)
    
    // Should contain setup instructions
    const instructionCard = cards.find((card: any) => 
      card.question.includes('How to Enable') || card.answer.includes('Settings')
    )
    
    expect(instructionCard).toBeDefined()
    expect(instructionCard.answer).toContain('AnkiDroid')
    expect(instructionCard.answer).toContain('Settings')
  })

  test('should not throw unhandled exceptions', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    // Test that the function doesn't throw, even on errors
    await expect(listCards()).resolves.not.toThrow()
    
    // Test with network errors
    mockInvoke.mockRejectedValueOnce(new Error('Network error'))
    await expect(listCards()).resolves.not.toThrow()
    
    // Test with permission errors
    mockInvoke.mockRejectedValueOnce(new Error('SecurityException: Permission denied'))
    await expect(listCards()).resolves.not.toThrow()
  })

  test('should handle various error scenarios gracefully', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    // Test different error scenarios
    const errorScenarios = [
      'SecurityException: Permission denied',
      'ContentProviderNotFoundException',
      'NetworkException: No connection',
      'Unknown error'
    ]
    
    for (const errorMessage of errorScenarios) {
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))
      
      const result = await listCards()
      expect(result).toBeDefined()
      
      // Should still return valid JSON
      const cards = JSON.parse(result)
      expect(Array.isArray(cards)).toBe(true)
      expect(cards.length).toBeGreaterThan(0)
      
      // Should contain error information
      const errorCard = cards[0]
      expect(errorCard.deck).toBe('Error')
    }
  })
})

describe('Crash Prevention Tests', () => {
  test('should never return undefined or null', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    // Even with severe errors, should return a string
    mockInvoke.mockRejectedValueOnce(new Error('FATAL ERROR'))
    
    const result = await listCards()
    expect(result).toBeDefined()
    expect(typeof result).toBe('string')
    expect(result.length).toBeGreaterThan(0)
  })

  test('should handle malformed responses', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    // Test with malformed responses
    mockInvoke.mockResolvedValueOnce('invalid json{')
    
    const result = await listCards()
    expect(result).toBeDefined()
    
    // Should still be valid JSON
    expect(() => JSON.parse(result)).not.toThrow()
  })

  test('should handle timeout scenarios', async () => {
    const { listCards } = await import('tauri-plugin-ankidroid-js')
    
    // Simulate timeout
    mockInvoke.mockImplementationOnce(() => 
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Timeout')), 100)
      )
    )
    
    await expect(listCards()).resolves.not.toThrow()
  })
})