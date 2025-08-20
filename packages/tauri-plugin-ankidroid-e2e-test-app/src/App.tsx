import { useState, useEffect } from 'react';
import { listCards, createCard, getDecks, parseCards } from 'tauri-plugin-ankidroid-js';
import './App.css';

function App() {
  const [cardsData, setCardsData] = useState('');
  const [cardsLoading, setCardsLoading] = useState(false);
  const [createLoading, setCreateLoading] = useState(false);

  // Form state for creating cards
  const [front, setFront] = useState('Test HMR - Front');
  const [back, setBack] = useState('Test HMR - Back');
  const [selectedDeckId, setSelectedDeckId] = useState<number | null>(null);
  const [tags, setTags] = useState('e2e-test');
  const [createResult, setCreateResult] = useState('');

  // Deck list
  const [decks, setDecks] = useState<Array<{ id: number; name: string }>>([]);

  async function getCards() {
    setCardsLoading(true);
    try {
      const response = await listCards();

      // Validate that response is valid JSON
      try {
        JSON.parse(response);
        setCardsData(response);
      } catch (parseError) {
        console.warn('Invalid JSON response from listCards:', parseError);
        setCardsData(
          JSON.stringify(
            [
              {
                id: 0,
                front: 'Invalid JSON Response',
                back: `Raw response: ${response}`,
                deck: 'Error',
                tags: '',
                note: 'The response was not valid JSON',
              },
            ],
            null,
            2
          )
        );
      }
    } catch (error) {
      console.error('Error calling listCards:', error);
      setCardsData(
        JSON.stringify(
          [
            {
              id: 0,
              front: 'Failed to load cards',
              back: `Error: ${error}`,
              deck: 'Error',
              tags: '',
              note: 'Please check if AnkiDroid plugin is working correctly',
            },
          ],
          null,
          2
        )
      );
    } finally {
      setCardsLoading(false);
    }
  }

  async function handleCreateCard() {
    console.log('🟢 handleCreateCard called!', { front, back, selectedDeckId, tags });

    if (!front || !back) {
      setCreateResult('Please enter both front and back text');
      console.log('🟢 Missing front or back text');
      return;
    }

    setCreateLoading(true);
    setCreateResult('');

    try {
      console.log('🟢 Calling createCard...');
      // Find deck name from ID
      const selectedDeck = decks.find(d => d.id === selectedDeckId);
      const deckName = selectedDeck ? selectedDeck.name : 'Default';
      const result = await createCard(front, back, deckName, tags);
      console.log('🟢 createCard result:', result);

      if (result.success) {
        setCreateResult(`✅ Card created successfully! Note ID: ${result.noteId}`);
        // Clear form
        setFront('');
        setBack('');
        // Refresh card list
        await getCards();
      } else {
        setCreateResult(`❌ Failed to create card: ${result.error || 'Unknown error'}`);
      }
    } catch (error) {
      console.error('Error creating card:', error);
      setCreateResult(`❌ Error: ${error}`);
    } finally {
      setCreateLoading(false);
    }
  }

  async function loadDecks() {
    try {
      const deckList = await getDecks();
      setDecks(deckList);
      if (deckList.length > 0 && selectedDeckId === null) {
        setSelectedDeckId(deckList[0].id);
      }
    } catch (error) {
      console.error('Error loading decks:', error);
    }
  }

  // Load decks and cards on mount
  useEffect(() => {
    loadDecks();
    getCards();
  }, []);

  return (
    <div className="container">
      <h1>🃏 AnkiDroid E2E Test App</h1>

      <div className="section">
        <h2>📋 Your AnkiDroid Cards</h2>
        <div className="button-row">
          <button onClick={getCards} disabled={cardsLoading}>
            {cardsLoading ? '⏳ Loading Cards...' : '🔄 Read AnkiDroid Cards'}
          </button>
          <button onClick={loadDecks}>📂 Load Decks</button>
        </div>

        {decks.length > 0 && (
          <div className="decks-list">
            <h3>Available Decks:</h3>
            <ul>
              {decks.map((d) => (
                <li key={d.id}>
                  {d.name} (ID: {d.id})
                </li>
              ))}
            </ul>
          </div>
        )}

        {cardsData && (
          <div className="cards-container">
            <div className="cards-grid">
              {(() => {
                try {
                  const cards = parseCards(cardsData);
                  return cards.map((card, index) => (
                    <div key={card.id || index} className="card-item">
                      <div className="card-front">
                        <strong>Q:</strong> {card.front}
                      </div>
                      <div className="card-back">
                        <strong>A:</strong> {card.back}
                      </div>
                      <div className="card-meta">
                        <span className="card-deck">📚 {card.deck}</span>
                        {card.tags && <span className="card-tags">🏷️ {card.tags}</span>}
                      </div>
                    </div>
                  ));
                } catch {
                  return (
                    <div className="json-display">
                      <pre>{cardsData}</pre>
                    </div>
                  );
                }
              })()}
            </div>
          </div>
        )}
      </div>

      <div className="section">
        <h2>📝 Create New Card</h2>
        <div className="form">
          <div className="form-group">
            <label htmlFor="front">Front (Question):</label>
            <input
              id="front"
              type="text"
              value={front}
              onChange={(e) => setFront(e.target.value)}
              placeholder="Enter question..."
              className="input-field"
            />
          </div>

          <div className="form-group">
            <label htmlFor="back">Back (Answer):</label>
            <input
              id="back"
              type="text"
              value={back}
              onChange={(e) => setBack(e.target.value)}
              placeholder="Enter answer..."
              className="input-field"
            />
          </div>

          <div className="form-group">
            <label htmlFor="deck">Deck:</label>
            <select
              id="deck"
              value={selectedDeckId || ''}
              onChange={(e) => setSelectedDeckId(e.target.value ? Number(e.target.value) : null)}
              className="input-field"
            >
              <option value="">Select a deck...</option>
              {decks.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name} (ID: {d.id})
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="tags">Tags:</label>
            <input
              id="tags"
              type="text"
              value={tags}
              onChange={(e) => setTags(e.target.value)}
              placeholder="Tags (optional)..."
              className="input-field"
            />
          </div>

          <button onClick={handleCreateCard} disabled={createLoading} className="create-button">
            {createLoading ? '⏳ Creating...' : '➕ Create Card'}
          </button>

          {createResult && <div className="result-message">{createResult}</div>}
        </div>
      </div>
    </div>
  );
}

export default App;
