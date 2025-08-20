import { expect } from 'chai';

describe('Error Handling E2E Test', () => {
  let webviewContext = null;

  before(async () => {
    console.log('Setting up error handling test...');
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

  it('should handle invalid card creation gracefully', async () => {
    console.log('Testing error handling for invalid card creation...');
    
    // Try to create a card with empty fields (should show error toast)
    const frontInput = await $('#front');
    const backInput = await $('#back');
    
    // Clear all fields to force an error
    await frontInput.clearValue();
    await backInput.clearValue();
    
    // Try to create the card
    const createButton = await $('button*=Create Card');
    const isEnabled = await createButton.isEnabled();
    expect(isEnabled).to.be.true;
    
    console.log('Clicking Create Card button with empty fields...');
    await createButton.click();
    
    // Wait for error handling to complete
    await driver.pause(3000);
    
    // Check for error message in the result area
    const resultMessage = await $('.result-message');
    const isResultVisible = await resultMessage.isDisplayed().catch(() => false);
    
    if (isResultVisible) {
      const resultText = await resultMessage.getText();
      console.log('Result message:', resultText);
      
      // Should show an error message, not crash the app
      expect(resultText).to.include('❌'); // Should have error indicator
      console.log('✅ Error message displayed correctly');
    } else {
      console.log('No result message visible - checking if app is still responsive');
    }
    
    // Most importantly, verify the app is still responsive
    const isAppResponsive = await frontInput.isDisplayed();
    expect(isAppResponsive).to.be.true;
    console.log('✅ App remains responsive after error');
  });

  it('should handle deck loading errors gracefully', async () => {
    console.log('Testing error handling for deck loading...');
    
    // Click the Load Decks button - this might fail if AnkiDroid is not properly set up
    const loadDecksButton = await $('button*=Load Decks');
    await loadDecksButton.click();
    console.log('Clicked Load Decks button');
    
    // Wait for response
    await driver.pause(2000);
    
    // Check if the app is still responsive regardless of success/failure
    const deckSelector = await $('#deck');
    const isSelectVisible = await deckSelector.isDisplayed();
    expect(isSelectVisible).to.be.true;
    
    // Check if we have options (could be default or error handling)
    const options = await $$('#deck option');
    console.log(`Found ${options.length} deck options`);
    
    // Should have at least the default/placeholder option
    expect(options.length).to.be.greaterThan(0);
    console.log('✅ Deck selector remains functional');
  });

  it('should handle card listing errors gracefully', async () => {
    console.log('Testing error handling for card listing...');
    
    // Click the Read Cards button
    const readCardsButton = await $('button*=Read AnkiDroid Cards');
    await readCardsButton.click();
    console.log('Clicked Read Cards button');
    
    // Wait for response
    await driver.pause(3000);
    
    // Check if the app is still responsive
    const isButtonVisible = await readCardsButton.isDisplayed();
    expect(isButtonVisible).to.be.true;
    
    // The card list area should still be present (even if empty or showing error)
    const cardList = await $('.card-list, .cards-container, #cards').catch(() => null);
    if (cardList) {
      const isListVisible = await cardList.isDisplayed();
      if (isListVisible) {
        console.log('✅ Card list area remains visible');
      }
    }
    
    console.log('✅ App remains responsive after card listing operation');
  });

  it('should maintain form state after errors', async () => {
    console.log('Testing form state preservation after errors...');
    
    // Fill in some test data
    const testFront = 'Error Test Question';
    const testBack = 'Error Test Answer';
    
    const frontInput = await $('#front');
    const backInput = await $('#back');
    
    await frontInput.setValue(testFront);
    await backInput.setValue(testBack);
    
    // Verify the data was entered
    const frontValue = await frontInput.getValue();
    const backValue = await backInput.getValue();
    
    expect(frontValue).to.equal(testFront);
    expect(backValue).to.equal(testBack);
    
    console.log('✅ Form accepts and retains input data');
    
    // Try some operation that might fail
    const createButton = await $('button*=Create Card');
    await createButton.click();
    await driver.pause(2000);
    
    // Check that we can still interact with the form
    await frontInput.clearValue();
    await frontInput.setValue('Updated test');
    
    const updatedValue = await frontInput.getValue();
    expect(updatedValue).to.equal('Updated test');
    
    console.log('✅ Form remains interactive after potential errors');
  });

  it('should not crash on rapid button clicks', async () => {
    console.log('Testing rapid button clicking (stress test)...');
    
    // Get all interactive buttons
    const createButton = await $('button*=Create Card');
    const loadDecksButton = await $('button*=Load Decks');
    const readCardsButton = await $('button*=Read AnkiDroid Cards');
    
    // Click buttons rapidly to test for race conditions
    for (let i = 0; i < 3; i++) {
      console.log(`Rapid click iteration ${i + 1}`);
      
      if (await loadDecksButton.isEnabled()) {
        await loadDecksButton.click();
      }
      
      if (await readCardsButton.isEnabled()) {
        await readCardsButton.click();
      }
      
      if (await createButton.isEnabled()) {
        await createButton.click();
      }
      
      // Small pause between iterations
      await driver.pause(500);
    }
    
    // Wait for all operations to settle
    await driver.pause(3000);
    
    // Verify the app is still responsive
    const frontInput = await $('#front');
    const isInputResponsive = await frontInput.isEnabled();
    expect(isInputResponsive).to.be.true;
    
    console.log('✅ App survives rapid button clicking without crashing');
  });

  after(async () => {
    // Switch back to native context
    if (webviewContext) {
      await driver.switchContext('NATIVE_APP');
      console.log('Switched back to native context');
    }
  });
});