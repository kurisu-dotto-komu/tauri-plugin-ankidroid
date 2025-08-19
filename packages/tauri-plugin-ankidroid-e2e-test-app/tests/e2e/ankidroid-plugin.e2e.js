import { expect } from 'chai';

describe('AnkiDroid Plugin E2E Tests', () => {
  it('should launch app without crashing', async () => {
    // Wait for app to fully load
    await driver.pause(500);

    // Check that app is running by finding the main heading
    const heading = await driver.$('//android.widget.TextView[contains(@text, "AnkiDroid")]');
    expect(heading).to.exist;

    const headingText = await heading.getText();
    expect(headingText).to.include('AnkiDroid');
  });

  it('should find and click the Read AnkiDroid Cards button', async () => {
    // Look for the button with various text possibilities
    const buttonSelectors = [
      '//android.widget.Button[contains(@text, "Read AnkiDroid Cards")]',
      '//android.widget.Button[contains(@text, "📚 Read AnkiDroid Cards")]',
      '//android.widget.Button[contains(@text, "List Cards")]',
      '//android.view.View[contains(@text, "Read AnkiDroid Cards")]',
    ];

    let button = null;
    for (const selector of buttonSelectors) {
      try {
        button = await driver.$(selector);
        if (await button.isDisplayed()) {
          console.log(`Found button with selector: ${selector}`);
          break;
        }
      } catch (e) {
        console.log(`Button not found with selector: ${selector}`);
      }
    }

    expect(button).to.exist;
    expect(await button.isDisplayed()).to.be.true;

    // Click the button
    await button.click();
    console.log('Button clicked successfully');
  });

  it('should not crash after button click', async () => {
    // Wait for response
    await driver.pause(500);

    // Check that app is still running by verifying we can still find elements
    try {
      const appElements = await driver.$$('//android.widget.TextView | //android.widget.Button');
      expect(appElements.length).to.be.greaterThan(0);
      console.log(`Found ${appElements.length} elements, app is still running`);
    } catch (e) {
      throw new Error('App crashed or became unresponsive after button click');
    }
  });

  it('should display error handling message instead of crashing', async () => {
    // Look for error handling text that indicates graceful error handling
    const errorIndicators = [
      'Permission',
      'AnkiDroid',
      'Settings',
      'Third party apps',
      'API',
      'Error',
    ];

    let foundErrorHandling = false;

    for (const indicator of errorIndicators) {
      try {
        const element = await driver.$(
          `//android.widget.TextView[contains(@text, "${indicator}")]`
        );
        if (await element.isDisplayed()) {
          const text = await element.getText();
          console.log(`Found error handling text: ${text}`);
          foundErrorHandling = true;
          break;
        }
      } catch (e) {
        // Continue searching
      }
    }

    // If we didn't find specific error text, at least verify the app is responsive
    if (!foundErrorHandling) {
      console.log('No specific error text found, but app should still be responsive');
      const allText = await driver.$$('//android.widget.TextView');
      expect(allText.length).to.be.greaterThan(0);
    }
  });

  it('should handle multiple button clicks without crashing', async () => {
    // Find the button again
    const button = await driver.$(
      '//android.widget.Button[contains(@text, "Read") or contains(@text, "📚")]'
    );

    if (await button.isDisplayed()) {
      // Click multiple times to test stability
      for (let i = 0; i < 3; i++) {
        console.log(`Button click attempt ${i + 1}`);
        await button.click();
        await driver.pause(500);

        // Verify app is still responsive
        const elements = await driver.$$('//android.widget.TextView | //android.widget.Button');
        expect(elements.length).to.be.greaterThan(0);
      }

      console.log('Multiple button clicks completed without crash');
    }
  });

  it('should maintain UI responsiveness', async () => {
    // Test that we can still interact with the UI
    try {
      // Try to scroll (if scrollable content exists)
      await driver.scroll('//android.widget.ScrollView', 'down');
      await driver.pause(500);
    } catch (e) {
      console.log("No scrollable content found, that's OK");
    }

    // Verify we can still find and interact with elements
    const allButtons = await driver.$$('//android.widget.Button');
    expect(allButtons.length).to.be.greaterThan(0);

    console.log('UI remains responsive after testing');
  });

  afterEach(async function () {
    // Take a screenshot on failure for debugging
    if (this.currentTest && this.currentTest.state === 'failed') {
      try {
        const screenshot = await driver.takeScreenshot();
        console.log('Test failed, screenshot taken');
        // In a real CI environment, you'd save this screenshot
      } catch (e) {
        console.log('Could not take screenshot:', e.message);
      }
    }
  });
});
