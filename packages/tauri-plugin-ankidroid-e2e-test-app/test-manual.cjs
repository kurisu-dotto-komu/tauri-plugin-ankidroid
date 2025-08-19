const { remote } = require('webdriverio');

(async () => {
  const driver = await remote({
    capabilities: {
      platformName: 'Android',
      'appium:automationName': 'UiAutomator2',
      'appium:deviceName': 'emulator-5554',
      'appium:appPackage': 'com.tauri.ankidroid.demo',
      'appium:appActivity': '.MainActivity',
      'appium:noReset': true,
    },
    hostname: 'localhost',
    port: 4723,
    path: '/',
  });

  console.log('App started, waiting for load...');
  await driver.pause(2000);

  // Try to fill in the form
  const inputs = await driver.$$('//android.widget.EditText');
  console.log(`Found ${inputs.length} input fields`);

  if (inputs.length >= 2) {
    await inputs[0].setValue('Test Front');
    await inputs[1].setValue('Test Back');
    console.log('Filled in form fields');
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

  // Find and click Create button
  const createButton = await driver.$('//android.widget.TextView[contains(@text, "Create")]');
  if (await createButton.isDisplayed()) {
    console.log('Clicking Create button...');
    await createButton.click();
    console.log('Button clicked!');

    // Wait for result
    await driver.pause(2000);

    // Check for any result message
    const allTextViews = await driver.$$('//android.widget.TextView');
    console.log('\nAll text on screen:');
    for (let i = 0; i < allTextViews.length; i++) {
      try {
        const text = await allTextViews[i].getText();
        if (text && text.length > 0) {
          console.log(`  ${i}: "${text}"`);
        }
      } catch (e) {
        // Skip
      }
    }
  }

  await driver.pause(1000);
  await driver.deleteSession();
})();
