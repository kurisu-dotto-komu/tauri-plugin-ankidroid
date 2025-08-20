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

  it('should click a button and check for results', async () => {
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
    } else {
      console.log('ℹ️ Decks list not visible (might be empty or need permissions)');
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