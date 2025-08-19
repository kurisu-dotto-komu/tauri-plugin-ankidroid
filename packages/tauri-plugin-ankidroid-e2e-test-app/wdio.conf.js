export const config = {
    specs: ['./tests/e2e/**/*.js'],
    maxInstances: 1,
    capabilities: [{
        'tauri:options': {
            application: '../src-tauri/target/release/demo-app'
        }
    }],
    services: [
        ['tauri', {
            tauriDriverPath: 'tauri-driver'
        }]
    ],
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000
    },
    logLevel: 'info',
    bail: 0,
    baseUrl: 'http://localhost',
    waitforTimeout: 10000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    
    before: function () {
        // Browser setup if needed
    },
    
    after: function () {
        // Browser cleanup if needed
    }
}