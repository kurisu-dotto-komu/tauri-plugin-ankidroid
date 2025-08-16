import { useState } from 'react'
import { listCards } from 'tauri-plugin-ankidroid-api'
import './App.css'

function App() {
  const [cardsData, setCardsData] = useState('')
  const [cardsLoading, setCardsLoading] = useState(false)

  async function getCards() {
    setCardsLoading(true)
    try {
      const response = await listCards()
      
      // Validate that response is valid JSON
      try {
        JSON.parse(response)
        setCardsData(response)
      } catch (parseError) {
        console.warn('Invalid JSON response from listCards:', parseError)
        setCardsData(JSON.stringify([{
          id: 0,
          question: "Invalid JSON Response",
          answer: `Raw response: ${response}`,
          deck: "Error",
          note: "The response was not valid JSON"
        }], null, 2))
      }
    } catch (error) {
      console.error('Error calling listCards:', error)
      setCardsData(JSON.stringify([{
        id: 0,
        question: "Failed to load cards",
        answer: `Error: ${error}`,
        deck: "Error",
        note: "Please check if AnkiDroid plugin is working correctly"
      }], null, 2))
    } finally {
      setCardsLoading(false)
    }
  }

  return (
    <div className="container">
      <h1>🃏 AnkiDroid Card Reader</h1>
      
      <div className="button-row">
        <button onClick={getCards} disabled={cardsLoading}>
          {cardsLoading ? '⏳ Loading Cards...' : '📚 Read AnkiDroid Cards'}
        </button>
      </div>
      
      {cardsData && (
        <div className="cards-container">
          <h3 className="cards-title">📋 Your AnkiDroid Cards</h3>
          <div className="json-display">
            {(() => {
              try {
                return JSON.stringify(JSON.parse(cardsData), null, 2)
              } catch {
                return cardsData
              }
            })()}
          </div>
        </div>
      )}
    </div>
  )
}

export default App