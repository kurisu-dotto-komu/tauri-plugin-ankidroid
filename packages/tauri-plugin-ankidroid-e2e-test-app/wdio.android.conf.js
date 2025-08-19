export const config = {
    specs: ['./tests/e2e/**/*.js'],
    maxInstances: 1,
    capabilities: [{
        platformName: 'Android',
        'appium:automationName': 'UiAutomator2',
        'appium:deviceName': 'emulator-5554',
        'appium:app': '/workspaces/tauri-plugin-ankidroid/packages/demo-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk',
        'appium:appPackage': 'com.tauri.ankidroid.demo',
        'appium:appActivity': '.MainActivity',
        'appium:noReset': false,
        'appium:newCommandTimeout': 60000,
        'appium:autoGrantPermissions': true
    }],
    services: [
        ['appium', {
            command: 'appium',
            args: {
                address: 'localhost',
                port: 4723,
                logLevel: 'info'
            }
        }]
    ],
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 120000
    },
    logLevel: 'info',
    bail: 0,
    waitforTimeout: 30000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    
    before: async function () {
        // Wait for app to load
        await driver.pause(3000)
    },
    
    after: async function () {
        // Cleanup if needed
    }
}