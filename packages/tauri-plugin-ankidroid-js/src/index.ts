import { invoke } from '@tauri-apps/api/core'

export interface HelloResponse {
    value: string
}

/**
 * Sends a hello message to the AnkiDroid plugin
 * @param name The name to greet
 * @returns A greeting message from the plugin
 */
export async function hello(name: string): Promise<string> {
    const response = await invoke<HelloResponse>('plugin:ankidroid|hello', { name })
    return response.value
}

/**
 * Gets a list of cards from AnkiDroid
 * @returns A JSON string containing card data
 */
export async function listCards(): Promise<string> {
    return await invoke<string>('plugin:ankidroid|list_cards')
}

// Future AnkiDroid API functions will be added here
export interface AddNoteRequest {
    front: string
    back: string
    deck?: string
}

export interface AddNoteResponse {
    noteId: number
    success: boolean
}