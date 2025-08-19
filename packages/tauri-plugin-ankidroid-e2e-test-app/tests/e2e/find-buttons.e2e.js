import { expect } from 'chai';

describe('Debug: Find Buttons and Inputs', () => {
  before(async () => {
    await driver.pause(500);
    console.log('🔍 Looking for buttons and inputs...');
  });

  it('should find all interactive elements', async () => {
    // Scroll down to see if there are more elements
    console.log('\n📜 Checking entire page for elements...\n');

    // Find ALL elements on the page
    const allElements = await driver.$$('//*');
    console.log(`Total elements on page: ${allElements.length}`);

    // Find elements that might be buttons (looking for emoji patterns)
    const potentialButtons = [];
    const foundInputs = [];

    for (let i = 0; i < allElements.length; i++) {
      const element = allElements[i];
      try {
        const text = await element.getText();
        const className = await element.getAttribute('className');

        // Check if it's an input
        if (className && className.includes('EditText')) {
          foundInputs.push({
            index: i,
            text,
            className,
          });
        }

        // Check if text contains emojis or button-like text
        if (
          text &&
          (text.includes('➕') ||
            text.includes('🔄') ||
            text.includes('Create') ||
            text.includes('Read') ||
            text.includes('Load') ||
            text.includes('📂') ||
            text.includes('Card'))
        ) {
          const clickable = await element.getAttribute('clickable');
          const bounds = await element.getAttribute('bounds');
          potentialButtons.push({
            index: i,
            text: text.substring(0, 50),
            className,
            clickable,
            bounds,
          });
        }
      } catch (e) {
        // Element might not have these attributes
      }
    }

    console.log('\n📝 Found Inputs:');
    foundInputs.forEach((input) => {
      console.log(`  Input ${input.index}: "${input.text}" (${input.className})`);
    });

    console.log('\n🔘 Potential Buttons:');
    potentialButtons.forEach((btn) => {
      console.log(
        `  Element ${btn.index}: "${btn.text}" (${btn.className}, clickable=${btn.clickable}, bounds=${btn.bounds})`
      );
    });

    // Try to find the 4th input (tags) by scrolling
    console.log('\n📜 Scrolling to find more elements...');

    try {
      // Try to scroll down
      await driver.execute('mobile: scroll', { direction: 'down' });
      await driver.pause(500);

      // Check for new EditText elements after scroll
      const inputsAfterScroll = await driver.$$('//android.widget.EditText');
      console.log(`Found ${inputsAfterScroll.length} EditText elements after scrolling`);

      if (inputsAfterScroll.length > 3) {
        const fourthInput = inputsAfterScroll[3];
        const text = await fourthInput.getText();
        console.log(`Fourth input found: "${text}"`);
      }
    } catch (e) {
      console.log('Could not scroll:', e.message);
    }

    // Try finding elements by partial text matching
    console.log('\n🔎 Looking for elements with button-like content...');

    const buttonPatterns = [
      '//*[contains(@content-desc, "Create")]',
      '//*[contains(@content-desc, "button")]',
      '//*[@content-desc]',
      '//*[contains(@text, "➕")]',
      '//android.view.View[contains(@text, "➕")]',
      '//android.widget.TextView[contains(@text, "➕")]',
    ];

    for (const pattern of buttonPatterns) {
      try {
        const elements = await driver.$$(pattern);
        if (elements.length > 0) {
          console.log(`Pattern "${pattern}" found ${elements.length} elements`);
          for (let i = 0; i < Math.min(2, elements.length); i++) {
            try {
              const text = await elements[i].getText();
              const contentDesc = await elements[i].getAttribute('content-desc');
              console.log(`  - Element: text="${text}", content-desc="${contentDesc}"`);
            } catch (e) {
              // Continue
            }
          }
        }
      } catch (e) {
        // Pattern might not be valid
      }
    }

    // Get page source for analysis
    console.log('\n📄 Getting page source snippet...');
    const pageSource = await driver.getPageSource();

    // Find button-related content in page source
    if (pageSource.includes('➕')) {
      console.log('✅ Found ➕ emoji in page source');
    }
    if (pageSource.includes('🔄')) {
      console.log('✅ Found 🔄 emoji in page source');
    }
    if (pageSource.includes('Create Card')) {
      console.log('✅ Found "Create Card" text in page source');
    }
    if (pageSource.includes('Read AnkiDroid')) {
      console.log('✅ Found "Read AnkiDroid" text in page source');
    }

    // Look for the actual structure around buttons
    const createCardIndex = pageSource.indexOf('Create Card');
    if (createCardIndex > -1) {
      const snippet = pageSource.substring(
        Math.max(0, createCardIndex - 200),
        createCardIndex + 200
      );
      console.log('\nHTML around "Create Card":', snippet.replace(/</g, '\n<'));
    }

    expect(true).to.be.true;
  });
});
