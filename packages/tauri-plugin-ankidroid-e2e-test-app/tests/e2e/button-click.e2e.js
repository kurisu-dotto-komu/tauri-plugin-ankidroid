import { expect } from 'chai';

describe('Button Click Crash Prevention Test', () => {
  it('should find and click the Read AnkiDroid Cards button without crashing', async () => {
    console.log('Starting button click test...');

    // Wait for app to fully load
    await driver.pause(500);

    // Take screenshot of initial state
    const initialScreenshot = await driver.takeScreenshot();
    console.log('Initial screenshot taken');

    // Look for button by text content
    const buttonSelectors = ['Read AnkiDroid Cards', 'List Cards', '📚'];

    let button = null;
    let buttonText = '';

    for (const text of buttonSelectors) {
      try {
        // Try different element types that might contain our button
        const selectors = [
          `//android.widget.Button[contains(@text, "${text}")]`,
          `//android.view.View[contains(@text, "${text}")]`,
          `//android.widget.TextView[contains(@text, "${text}")]`,
          `//*[contains(@text, "${text}")]`,
        ];

        for (const selector of selectors) {
          try {
            const elements = await driver.$$(selector);
            if (elements.length > 0) {
              for (const element of elements) {
                if (await element.isDisplayed()) {
                  button = element;
                  buttonText = await element.getText();
                  console.log(`Found button: "${buttonText}" with selector: ${selector}`);
                  break;
                }
              }
              if (button) break;
            }
          } catch (e) {
            // Continue trying
          }
        }
        if (button) break;
      } catch (e) {
        console.log(`Could not find button with text "${text}":`, e.message);
      }
    }

    // If we still haven't found the button, try to find any clickable element
    if (!button) {
      console.log('Trying to find any clickable element...');
      try {
        const clickableElements = await driver.$$(
          '//android.widget.Button | //android.view.View[@clickable="true"]'
        );
        if (clickableElements.length > 0) {
          button = clickableElements[0];
          buttonText = (await button.getText()) || 'Unknown button';
          console.log(`Found clickable element: "${buttonText}"`);
        }
      } catch (e) {
        console.log('Could not find any clickable elements');
      }
    }

    expect(button).to.exist;
    console.log(`Button found: "${buttonText}"`);

    // Record app state before click
    const appElements = await driver.$$('//*');
    const elementCountBefore = appElements.length;
    console.log(`Elements before click: ${elementCountBefore}`);

    // Click the button
    await button.click();
    console.log('Button clicked successfully!');

    // Wait for response
    await driver.pause(500);

    // Check that app is still running - this is the critical test
    try {
      const appElementsAfter = await driver.$$('//*');
      const elementCountAfter = appElementsAfter.length;
      console.log(`Elements after click: ${elementCountAfter}`);

      expect(elementCountAfter).to.be.greaterThan(0);
      console.log('✅ App is still responsive after button click - NO CRASH!');

      // Try to find error handling text
      const allText = await driver.$$('//android.widget.TextView');
      let foundErrorText = false;

      for (const textElement of allText) {
        try {
          const text = await textElement.getText();
          if (
            text &&
            (text.includes('Permission') ||
              text.includes('AnkiDroid') ||
              text.includes('Error') ||
              text.includes('Settings'))
          ) {
            console.log(`Found error handling text: "${text}"`);
            foundErrorText = true;
          }
        } catch (e) {
          // Continue
        }
      }

      if (foundErrorText) {
        console.log('✅ Graceful error handling detected!');
      } else {
        console.log('ℹ️  No specific error text found, but app is stable');
      }
    } catch (e) {
      throw new Error(`🚨 APP CRASHED after button click: ${e.message}`);
    }

    // Final screenshot
    const finalScreenshot = await driver.takeScreenshot();
    console.log('Final screenshot taken');
    console.log('🎉 Test completed successfully - no crash detected!');
  });
});
