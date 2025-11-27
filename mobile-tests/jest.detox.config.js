module.exports = {
  preset: 'ts-jest',
  testEnvironment: './environment',
  testTimeout: 120000,
  testMatch: ['<rootDir>/**/*.test.ts'],
  reporters: ['detox/runners/jest/reporter'],
  globalSetup: 'detox/runners/jest/globalSetup',
  globalTeardown: 'detox/runners/jest/globalTeardown',
  verbose: true,
};
