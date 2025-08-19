import { expect } from 'chai';

describe('Simple Create Test', () => {
  it('should attempt to create a card and check logs', async () => {
    console.log('Starting simple create test...');

    // Wait for app to load
    await driver.pause(500);

    // Fill in fields
    const inputs = await driver.$$('//android.widget.EditText');
    if (inputs.length >= 3) {
      await inputs[0].setValue('Test Front');
      await inputs[1].setValue('Test Back');
      await inputs[2].setValue('Test Deck');
    }

    // Scroll to find button
    const { width, height } = await driver.getWindowSize();
    await driver.performActions([
      {
        type: 'pointer',
        id: 'finger1',
        parameters: { pointerType: 'touch' },
        actions: [
          { type: 'pointerMove', duration: 0, x: width / 2, y: height * 0.6 },
          { type: 'pointerDown', button: 0 },
          { type: 'pause', duration: 200 },
          { type: 'pointerMove', duration: 500, x: width / 2, y: height * 0.3 },
          { type: 'pointerUp', button: 0 },
        ],
      },
    ]);
    await driver.pause(500);

    // Click create button
    const createButton = await driver.$('//android.widget.TextView[contains(@text, "Create")]');
    if (await createButton.isDisplayed()) {
      console.log('Clicking create button...');
      await createButton.click();
      await driver.pause(500);
    }

    // Check for any visible text
    const allTextViews = await driver.$$('//android.widget.TextView');
    console.log(`Found ${allTextViews.length} TextViews after click`);
    for (let i = 0; i < allTextViews.length; i++) {
      try {
        const text = await allTextViews[i].getText();
        if (text && text.length > 0) {
          console.log(`Text ${i}: "${text}"`);
        }
      } catch (e) {
        // Skip
      }
    }
  });
});
