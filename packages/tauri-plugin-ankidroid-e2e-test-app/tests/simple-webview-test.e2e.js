import { expect } from 'chai';

describe('Simple WebView Test with $() Selectors', () => {
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
  });

  it('should switch to webview and find the title using $()', async () => {
    // Switch to WebView context
    await driver.switchContext(webviewContext);
    console.log('Switched to WebView context');

    // Now we can use standard web selectors!
    const title = await $('h1');
    const titleText = await title.getText();
    
    console.log(`Title text: "${titleText}"`);
    expect(titleText).to.equal('🃏 AnkiDroid E2E Test App');
    
    console.log('✅ Successfully found title using $() selector');
  });

  it('should find and interact with form inputs using standard selectors', async () => {
    // Make sure we're in WebView context
    await driver.switchContext(webviewContext);

    // Find inputs by ID
    const frontInput = await $('#front');
    const backInput = await $('#back');
    
    // Clear and set values
    await frontInput.clearValue();
    await frontInput.setValue('Test Question from WebView');
    
    await backInput.clearValue(); 
    await backInput.setValue('Test Answer from WebView');
    
    // Verify the values were set
    const frontValue = await frontInput.getValue();
    const backValue = await backInput.getValue();
    
    expect(frontValue).to.equal('Test Question from WebView');
    expect(backValue).to.equal('Test Answer from WebView');
    
    console.log('✅ Successfully interacted with form inputs');
  });

  it('should find buttons using CSS selectors', async () => {
    await driver.switchContext(webviewContext);

    // Find all buttons
    const buttons = await $$('button');
    console.log(`Found ${buttons.length} buttons`);
    expect(buttons.length).to.be.greaterThan(0);

    // Find specific button by text content
    const readButton = await $('button*=Read AnkiDroid Cards');
    const isDisplayed = await readButton.isDisplayed();
    expect(isDisplayed).to.be.true;
    
    console.log('✅ Found Read AnkiDroid Cards button');
  });

  it('should use complex CSS selectors', async () => {
    await driver.switchContext(webviewContext);

    // Find section by heading
    const section = await $('.section');
    expect(await section.isDisplayed()).to.be.true;

    // Find form groups
    const formGroups = await $$('.form-group');
    console.log(`Found ${formGroups.length} form groups`);
    expect(formGroups.length).to.be.greaterThan(0);

    // Find by attribute selectors
    const frontLabel = await $('label[for="front"]');
    const labelText = await frontLabel.getText();
    expect(labelText).to.include('Front');
    
    console.log('✅ Complex CSS selectors working');
  });

  it('should test Load Decks functionality', async () => {
    await driver.switchContext(webviewContext);

    // Click the Load Decks button
    const loadDecksButton = await $('button*=Load Decks');
    await loadDecksButton.click();
    console.log('Clicked Load Decks button');

    // Wait a bit for the operation
    await driver.pause(1000);

    // Check if decks list appeared
    const decksList = await $('.decks-list');
    const isDecksListVisible = await decksList.isDisplayed().catch(() => false);
    
    if (isDecksListVisible) {
      const decksItems = await $$('.decks-list li');
      console.log(`✅ Found ${decksItems.length} decks`);
      
      // Check if deck select dropdown was populated
      const deckSelect = await $('#deck');
      const deckOptions = await $$('#deck option');
      console.log(`✅ Found ${deckOptions.length} deck options in dropdown`);
      expect(deckOptions.length).to.be.greaterThan(1); // At least "Select a deck..." + 1 deck
    } else {
      console.log('ℹ️ Decks list not visible (might be empty or need permissions)');
    }
  });

  it('should test Load Models functionality', async () => {
    await driver.switchContext(webviewContext);

    // Click the Load Models button
    const loadModelsButton = await $('button*=Load Models');
    await loadModelsButton.click();
    console.log('Clicked Load Models button');

    // Wait a bit for the operation
    await driver.pause(1000);

    // Check if models list appeared
    const modelsList = await $('.models-list');
    const isModelsListVisible = await modelsList.isDisplayed().catch(() => false);
    
    if (isModelsListVisible) {
      const modelsItems = await $$('.models-list li');
      console.log(`✅ Found ${modelsItems.length} models`);
      
      // Check if model select dropdown was populated
      const modelSelect = await $('#model');
      const modelOptions = await $$('#model option');
      console.log(`✅ Found ${modelOptions.length} model options in dropdown`);
      expect(modelOptions.length).to.be.greaterThan(1); // At least "Select a model..." + 1 model
    } else {
      console.log('ℹ️ Models list not visible (might be empty or no models found)');
      
      // Still check if dropdown exists
      const modelSelect = await $('#model');
      const isModelSelectVisible = await modelSelect.isDisplayed().catch(() => false);
      expect(isModelSelectVisible).to.be.true;
      console.log('✅ Model select dropdown is present');
    }
  });

  it('should test card creation form with deck and model selection', async () => {
    await driver.switchContext(webviewContext);

    // First load decks and models
    const loadDecksButton = await $('button*=Load Decks');
    await loadDecksButton.click();
    await driver.pause(500);
    
    const loadModelsButton = await $('button*=Load Models');
    await loadModelsButton.click();
    await driver.pause(500);

    // Fill out the form
    const frontInput = await $('#front');
    const backInput = await $('#back');
    const tagsInput = await $('#tags');
    
    await frontInput.clearValue();
    await frontInput.setValue('E2E Test Question');
    
    await backInput.clearValue();
    await backInput.setValue('E2E Test Answer');
    
    await tagsInput.clearValue();
    await tagsInput.setValue('e2e-test automated');

    // Check deck dropdown
    const deckSelect = await $('#deck');
    const deckOptions = await $$('#deck option');
    if (deckOptions.length > 1) {
      // Select the first actual deck (not the placeholder)
      await deckSelect.selectByIndex(1);
      console.log('✅ Selected a deck');
    }

    // Check model dropdown
    const modelSelect = await $('#model');
    const modelOptions = await $$('#model option');
    if (modelOptions.length > 1) {
      // Select the first actual model (not the placeholder)
      await modelSelect.selectByIndex(1);
      console.log('✅ Selected a model');
    }

    // Find and verify the Create Card button
    const createButton = await $('button*=Create Card');
    const isCreateButtonVisible = await createButton.isDisplayed();
    expect(isCreateButtonVisible).to.be.true;
    console.log('✅ Create Card button is ready');

    // Note: We don't actually click create to avoid polluting the user's AnkiDroid
    // In a real test environment, you would click and verify the result
  });

  it('should verify deck names are properly displayed', async () => {
    await driver.switchContext(webviewContext);

    // Load decks first
    const loadDecksButton = await $('button*=Load Decks');
    await loadDecksButton.click();
    await driver.pause(1000);

    // Check deck dropdown options
    const deckOptions = await $$('#deck option');
    if (deckOptions.length > 1) {
      // Get text from deck options
      for (let i = 1; i < Math.min(3, deckOptions.length); i++) {
        const optionText = await deckOptions[i].getText();
        console.log(`Deck option ${i}: ${optionText}`);
        
        // Verify it's not just showing "Deck [id]" pattern
        // A proper deck name should either have a real name or follow "Deck ID" format
        expect(optionText).to.not.be.empty;
        expect(optionText).to.include('Deck'); // Should contain 'Deck' or actual deck name
      }
      console.log('✅ Deck names are displaying correctly');
    }
  });

  it('should verify we can use jQuery-like selectors', async () => {
    await driver.switchContext(webviewContext);

    // Class selector
    const container = await $('.container');
    expect(await container.isDisplayed()).to.be.true;

    // ID selector  
    const tagsInput = await $('#tags');
    expect(await tagsInput.isDisplayed()).to.be.true;

    // Descendant selector
    const sectionHeading = await $('.section h2');
    expect(await sectionHeading.isDisplayed()).to.be.true;

    // Multiple elements
    const allInputs = await $$('input');
    console.log(`Found ${allInputs.length} input elements`);
    expect(allInputs.length).to.be.greaterThan(0);

    console.log('✅ All jQuery-like selectors working');
  });

  after(async () => {
    // Switch back to native context
    if (webviewContext) {
      await driver.switchContext('NATIVE_APP');
      console.log('Switched back to native context');
    }
  });
});