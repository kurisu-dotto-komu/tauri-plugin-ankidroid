import { expect } from 'chai';

describe('Create Card E2E Test', () => {
  let webviewContext = null;

  before(async () => {
    console.log('Waiting for app to load...');
    await driver.pause(3000);

    // Get and log all available contexts
    const contexts = await driver.getContexts();
    console.log('Available contexts:', contexts);

    // Find the webview context for our app specifically
    webviewContext = contexts.find(ctx => ctx.includes('WEBVIEW_com.tauri.ankidroid.demo'));
    
    if (!webviewContext) {
      // Fallback to any WEBVIEW context, but skip AnkiDroid's
      webviewContext = contexts.find(ctx => ctx.includes('WEBVIEW') && !ctx.includes('com.ichi2.anki'));
    }
    
    if (!webviewContext) {
      throw new Error('No WebView context found for the Tauri app. Make sure the app is running in a WebView.');
    }

    console.log(`Found WebView context: ${webviewContext}`);
    
    // Switch to WebView context
    await driver.switchContext(webviewContext);
    console.log('Switched to WebView context');
  });

  it('should load decks before creating a card', async () => {
    // Click the Load Decks button to ensure we have decks available
    const loadDecksButton = await $('button*=Load Decks');
    await loadDecksButton.click();
    console.log('Clicked Load Decks button');
    
    // Wait for decks to load
    await driver.pause(2000);
    
    // Check if deck selector has options
    const deckSelector = await $('#deck');
    const options = await $$('#deck option');
    console.log(`Found ${options.length} deck options`);
    
    // Should have at least 2 options (the placeholder and at least one deck)
    expect(options.length).to.be.greaterThan(1);
  });

  it('should fill in the card creation form', async () => {
    // Create unique timestamp for this test run
    const timestamp = Date.now();
    const uniqueFront = `E2E Test Question ${timestamp}`;
    const uniqueBack = `E2E Test Answer ${timestamp}`;
    
    // Store the values globally so other tests can use them
    global.testCardFront = uniqueFront;
    global.testCardBack = uniqueBack;
    
    // Fill in the front field
    const frontInput = await $('#front');
    await frontInput.clearValue();
    await frontInput.setValue(uniqueFront);
    
    // Fill in the back field
    const backInput = await $('#back');
    await backInput.clearValue();
    await backInput.setValue(uniqueBack);
    
    // Select the first available deck
    const deckSelector = await $('#deck');
    const deckOptions = await $$('#deck option');
    console.log(`Found ${deckOptions.length} deck options`);
    
    // Find the first non-empty option and select it
    let selectedDeckName = null;
    for (let i = 0; i < deckOptions.length; i++) {
      const optionValue = await deckOptions[i].getAttribute('value');
      const optionText = await deckOptions[i].getText();
      console.log(`Option ${i}: value="${optionValue}", text="${optionText}"`);
      
      if (optionValue && optionValue !== '' && optionValue !== null) {
        // Select this deck using the select element's method
        await deckSelector.selectByAttribute('value', optionValue);
        selectedDeckName = optionText;
        console.log(`Selected deck: ${optionText} (ID: ${optionValue})`);
        break;
      }
    }
    
    // Fill in tags
    const tagsInput = await $('#tags');
    await tagsInput.clearValue();
    await tagsInput.setValue('e2e-test automated');
    
    // Verify all values are set correctly
    const frontValue = await frontInput.getValue();
    const backValue = await backInput.getValue();
    const tagsValue = await tagsInput.getValue();
    
    expect(frontValue).to.equal(uniqueFront);
    expect(backValue).to.equal(uniqueBack);
    expect(tagsValue).to.equal('e2e-test automated');
    
    console.log('✅ Form filled successfully');
    if (selectedDeckName) {
      console.log(`Using deck: ${selectedDeckName}`);
    } else {
      console.log('Using default deck');
    }
  });

  it('should click the Create Card button', async () => {
    // Find and click the Create Card button
    const createButton = await $('button*=Create Card');
    const isEnabled = await createButton.isEnabled();
    expect(isEnabled).to.be.true;
    
    console.log('Clicking Create Card button...');
    await createButton.click();
    
    // Wait for the card creation process to complete
    await driver.pause(3000);
    
    console.log('✅ Create Card button clicked');
  });

  it('should verify the card was created successfully', async () => {
    // Check for success or error message
    const resultMessage = await $('.result-message');
    const isResultVisible = await resultMessage.isDisplayed().catch(() => false);
    
    if (isResultVisible) {
      const resultText = await resultMessage.getText();
      console.log('Result message:', resultText);
      
      // Check if it's a success message
      if (resultText.includes('✅')) {
        expect(resultText).to.include('Card created successfully');
        console.log('✅ Card created successfully!');
        
        // Extract the note ID if present
        const noteIdMatch = resultText.match(/Note ID: (\d+)/);
        if (noteIdMatch) {
          console.log(`Created Note ID: ${noteIdMatch[1]}`);
        }
      } else if (resultText.includes('❌')) {
        throw new Error(`Card creation failed: ${resultText}`);
      } else if (resultText.includes('📝')) {
        // Still creating - wait a bit more
        await driver.pause(2000);
        const updatedText = await resultMessage.getText();
        console.log('Updated result:', updatedText);
        if (updatedText.includes('❌')) {
          throw new Error(`Card creation failed: ${updatedText}`);
        }
      }
    } else {
      console.log('⚠️ No result message visible - checking form state');
    }
    
    // Check if the form was cleared (indicates success)
    const frontInput = await $('#front');
    const backInput = await $('#back');
    
    const frontValue = await frontInput.getValue();
    const backValue = await backInput.getValue();
    
    // If successful, the form should be cleared
    if (frontValue === '' && backValue === '') {
      console.log('✅ Form was cleared after creation (indicates success)');
    } else {
      console.log(`ℹ️ Form values - Front: "${frontValue}", Back: "${backValue}"`);
    }
  });

  it('should verify the new card appears in the card list', async () => {
    // Click the Read Cards button to refresh the list
    const readCardsButton = await $('button*=Read AnkiDroid Cards');
    await readCardsButton.click();
    console.log('Clicked Read Cards button to refresh list');
    
    // Wait for cards to load
    await driver.pause(3000);
    
    // Check if our card appears in the list
    const cardItems = await $$('.card-item');
    console.log(`Found ${cardItems.length} cards in the list`);
    
    // Look for our test card using the unique values
    const expectedFront = global.testCardFront || 'E2E Test Question';
    const expectedBack = global.testCardBack || 'E2E Test Answer';
    
    let foundTestCard = false;
    for (const card of cardItems) {
      const frontText = await card.$('.card-front').getText().catch(() => '');
      console.log(`Checking card: "${frontText}"`);
      
      if (frontText.includes(expectedFront)) {
        foundTestCard = true;
        console.log('✅ Found our E2E test card in the list!');
        
        // Verify the back text as well
        const backText = await card.$('.card-back').getText().catch(() => '');
        expect(backText).to.include(expectedBack);
        console.log('✅ Back text verified');
        
        // Check for tags
        const tagsElement = await card.$('.card-tags');
        if (tagsElement) {
          const tagsText = await tagsElement.getText().catch(() => '');
          if (tagsText) {
            console.log('Card tags:', tagsText);
            expect(tagsText).to.include('e2e-test');
          }
        }
        break;
      }
    }
    
    // This should fail if the card wasn't created
    expect(foundTestCard).to.be.true;
    if (!foundTestCard) {
      throw new Error('E2E test card was not found in the card list - card creation failed');
    }
  });

  after(async () => {
    // Switch back to native context
    if (webviewContext) {
      await driver.switchContext('NATIVE_APP');
      console.log('Switched back to native context');
    }
  });
});