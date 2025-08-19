import { invoke } from '@tauri-apps/api/core';

export interface HelloResponse {
  value: string;
}

export interface CreateCardRequest {
  front: string;
  back: string;
  deck?: string;
  tags?: string;
}

export interface CreateCardResponse {
  success: boolean;
  noteId?: number;
  message?: string;
  error?: string;
}

export interface Deck {
  id: number;
  name: string;
}

export interface Card {
  id: number;
  front: string;
  back: string;
  deck: string;
  tags: string;
  deckId?: number;
}

/**
 * Sends a hello message to the AnkiDroid plugin
 * @param name The name to greet
 * @returns A greeting message from the plugin
 */
export async function hello(name: string): Promise<string> {
  const response = await invoke<HelloResponse>('plugin:ankidroid|hello', { name });
  return response.value;
}

/**
 * Gets a list of cards from AnkiDroid
 * @returns A JSON string containing card data
 */
export async function listCards(): Promise<string> {
  return await invoke<string>('plugin:ankidroid|list_cards');
}

/**
 * Creates a new card in AnkiDroid
 * @param front The front side (question) of the card
 * @param back The back side (answer) of the card
 * @param deck Optional deck name (defaults to "Default")
 * @param tags Optional tags for the card
 * @returns Response with success status and note ID
 */
export async function createCard(
  front: string,
  back: string,
  deck?: string,
  tags?: string
): Promise<CreateCardResponse> {
  const response = await invoke<string>('plugin:ankidroid|create_card', {
    front,
    back,
    deck,
    tags,
  });
  return JSON.parse(response) as CreateCardResponse;
}

/**
 * Gets a list of available decks from AnkiDroid
 * @returns Array of deck objects with id and name
 */
export async function getDecks(): Promise<Deck[]> {
  const response = await invoke<string>('plugin:ankidroid|get_decks');
  return JSON.parse(response) as Deck[];
}

/**
 * Updates an existing card in AnkiDroid
 * @param noteId The ID of the note/card to update
 * @param front The new front side (question) of the card
 * @param back The new back side (answer) of the card
 * @param deck Optional new deck name
 * @param tags Optional new tags for the card
 * @returns Response with success status
 */
export async function updateCard(
  noteId: number,
  front: string,
  back: string,
  deck?: string,
  tags?: string
): Promise<CreateCardResponse> {
  const response = await invoke<string>('plugin:ankidroid|update_card', {
    noteId,
    front,
    back,
    deck,
    tags,
  });
  return JSON.parse(response) as CreateCardResponse;
}

/**
 * Deletes a card from AnkiDroid
 * @param noteId The ID of the note/card to delete
 * @returns Response with success status
 */
export async function deleteCard(noteId: number): Promise<CreateCardResponse> {
  const response = await invoke<string>('plugin:ankidroid|delete_card', {
    noteId,
  });
  return JSON.parse(response) as CreateCardResponse;
}

/**
 * Helper function to parse card list response
 * @param cardsJson JSON string from listCards
 * @returns Array of Card objects
 */
export function parseCards(cardsJson: string): Card[] {
  try {
    return JSON.parse(cardsJson) as Card[];
  } catch (error) {
    console.error('Failed to parse cards JSON:', error);
    return [];
  }
}
