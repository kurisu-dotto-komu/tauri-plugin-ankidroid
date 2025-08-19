export const config = {
  specs: ['./tests/e2e/**/*.js'],
  maxInstances: 1,
  baseUrl: 'http://127.0.0.1:4723',
  port: 4723,
  path: '/',
  capabilities: [
    {
      platformName: 'Android',
      'appium:automationName': 'UiAutomator2',
      'appium:deviceName': 'emulator-5554',
      // App is already installed, just launch it???
      'appium:appPackage': 'com.tauri.ankidroid.demo',
      'appium:appActivity': '.MainActivity',
      'appium:noReset': true,
      'appium:newCommandTimeout': 60000,
      'appium:autoGrantPermissions': true,
    },
  ],
  services: [
    [
      'appium',
      {
        command: 'appium',
        args: {
          address: 'localhost',
          port: 4723,
          logLevel: 'info',
        },
      },
    ],
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  logLevel: 'info',
  bail: 0,
  waitforTimeout: 30000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  before: async function () {
    // Wait for app to load
    await driver.pause(500);
  },

  after: async function () {
    // Cleanup if needed
  },
};
