import { expect } from 'chai';

describe('Debug: Find Available Elements', () => {
  before(async () => {
    // Wait for app to fully load
    await driver.pause(500);
    console.log('🔍 Starting element discovery...');
  });

  it('should find and log all available elements', async () => {
    console.log('\n📱 Searching for all element types...\n');

    // Try to find WebView
    try {
      const webviews = await driver.$$('//android.webkit.WebView');
      console.log(`Found ${webviews.length} WebView elements`);

      if (webviews.length > 0) {
        // Try to switch to WebView context
        const contexts = await driver.getContexts();
        console.log('Available contexts:', contexts);

        // Try to switch to webview context if available
        for (const context of contexts) {
          if (context.includes('WEBVIEW')) {
            console.log(`Switching to context: ${context}`);
            await driver.switchContext(context);

            // Now try to find HTML elements
            const inputs = await driver.$$('input');
            console.log(`Found ${inputs.length} input elements in WebView`);

            const buttons = await driver.$$('button');
            console.log(`Found ${buttons.length} button elements in WebView`);

            // Log all input details
            for (let i = 0; i < inputs.length; i++) {
              const input = inputs[i];
              try {
                const id = await input.getAttribute('id');
                const value = await input.getValue();
                const placeholder = await input.getAttribute('placeholder');
                console.log(
                  `Input ${i}: id="${id}", value="${value}", placeholder="${placeholder}"`
                );
              } catch (e) {
                console.log(`Input ${i}: Could not get attributes`);
              }
            }

            // Log all button details
            for (let i = 0; i < buttons.length; i++) {
              const button = buttons[i];
              try {
                const text = await button.getText();
                const className = await button.getAttribute('class');
                console.log(`Button ${i}: text="${text}", class="${className}"`);
              } catch (e) {
                console.log(`Button ${i}: Could not get attributes`);
              }
            }

            // Switch back to native context
            await driver.switchContext('NATIVE_APP');
          }
        }
      }
    } catch (e) {
      console.log('WebView search error:', e.message);
    }

    // Try native Android elements
    console.log('\n🤖 Searching for native Android elements...\n');

    const elementTypes = [
      'android.widget.EditText',
      'android.widget.Button',
      'android.widget.TextView',
      'android.view.View',
      'android.view.ViewGroup',
      'android.widget.FrameLayout',
    ];

    for (const elementType of elementTypes) {
      try {
        const elements = await driver.$$(`//${elementType}`);
        if (elements.length > 0) {
          console.log(`Found ${elements.length} ${elementType} elements`);

          // Get details of first few elements
          for (let i = 0; i < Math.min(3, elements.length); i++) {
            try {
              const text = await elements[i].getText();
              if (text) {
                console.log(
                  `  - Element ${i}: "${text.substring(0, 50)}${text.length > 50 ? '...' : ''}"`
                );
              }
            } catch (e) {
              // Element might not have text
            }
          }
        }
      } catch (e) {
        console.log(`Could not search for ${elementType}: ${e.message}`);
      }
    }

    // Try to find any clickable elements
    console.log('\n👆 Searching for clickable elements...\n');
    try {
      const clickables = await driver.$$('//*[@clickable="true"]');
      console.log(`Found ${clickables.length} clickable elements`);

      for (let i = 0; i < Math.min(5, clickables.length); i++) {
        try {
          const text = await clickables[i].getText();
          const className = await clickables[i].getAttribute('className');
          if (text || className) {
            console.log(
              `  - Clickable ${i}: class="${className}", text="${text ? text.substring(0, 30) : 'no text'}"`
            );
          }
        } catch (e) {
          // Continue
        }
      }
    } catch (e) {
      console.log('Could not find clickable elements:', e.message);
    }

    // Try to find elements with specific text
    console.log('\n📝 Searching for elements with specific text...\n');
    const searchTexts = ['Create', 'Card', 'Front', 'Back', 'Hello World', 'Read', 'AnkiDroid'];

    for (const searchText of searchTexts) {
      try {
        const elements = await driver.$$(`//*[contains(@text, "${searchText}")]`);
        if (elements.length > 0) {
          console.log(`Found ${elements.length} elements containing "${searchText}"`);
        }
      } catch (e) {
        // Continue
      }
    }

    // Take a screenshot for manual inspection
    console.log('\n📸 Taking screenshot for manual inspection...');
    const screenshot = await driver.takeScreenshot();
    console.log('Screenshot captured (base64 length:', screenshot.length, ')');

    // This test always passes - it's just for debugging
    expect(true).to.be.true;
  });
});
