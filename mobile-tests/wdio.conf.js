export const config = {
  runner: 'local',
  specs: ['./appium/**/*.test.ts'],
  maxInstances: 1,
  capabilities: [
    {
      platformName: process.env.APPIUM_PLATFORM ?? 'iOS',
      'appium:automationName': process.env.APPIUM_AUTOMATION ?? 'XCUITest',
      'appium:deviceName': process.env.APPIUM_DEVICE ?? 'iPhone 14',
      'appium:platformVersion': process.env.APPIUM_PLATFORM_VERSION ?? '17.0',
      'appium:app': process.env.APPIUM_APP ?? '/path/to/EclipseMobile.app',
      'appium:autoAcceptAlerts': true,
      'appium:newCommandTimeout': 120,
    },
  ],
  logLevel: 'info',
  bail: 0,
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },
  services: [['appium', { command: 'appium' }]],
  before: async function () {
    require('ts-node').register({
      transpileOnly: true,
    });
  },
};
