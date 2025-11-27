import { device, element, by, expect as detoxExpect } from 'detox';

describe('Mobile Authentication', () => {
  beforeAll(async () => {
    await device.launchApp({
      newInstance: true,
      permissions: { notifications: 'YES', camera: 'YES' },
    });
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  describe('Device Registration', () => {
    it('should display registration screen on first launch', async () => {
      await detoxExpect(element(by.id('registration-screen'))).toBeVisible();
    });

    it('should register device successfully', async () => {
      await element(by.id('device-name-input')).typeText('iPhone 15 Pro');
      await element(by.id('register-button')).tap();

      await waitFor(element(by.id('registration-success')))
        .toBeVisible()
        .withTimeout(5000);
    });

    it('should show device in registered devices list', async () => {
      await element(by.id('settings-tab')).tap();
      await element(by.id('devices-menu')).tap();

      await detoxExpect(element(by.text('iPhone 15 Pro'))).toBeVisible();
    });
  });

  describe('Biometric Authentication', () => {
    beforeEach(async () => {
      await device.setBiometricEnrollment(true);
    });

    it('should create biometric challenge', async () => {
      await element(by.id('login-button')).tap();
      await detoxExpect(element(by.id('biometric-prompt'))).toBeVisible();
    });

    it('should authenticate with successful biometric', async () => {
      await element(by.id('login-button')).tap();
      await device.matchBiometric();

      await waitFor(element(by.id('dashboard-screen')))
        .toBeVisible()
        .withTimeout(5000);
    });

    it('should fail authentication with failed biometric', async () => {
      await element(by.id('login-button')).tap();
      await device.unmatchBiometric();

      await detoxExpect(element(by.id('auth-error'))).toBeVisible();
    });

    it('should require biometric re-authentication after timeout', async () => {
      // Login first
      await element(by.id('login-button')).tap();
      await device.matchBiometric();

      // Simulate session timeout
      await device.sendToHome();
      await device.launchApp({ newInstance: false });

      // Should require biometric again
      await detoxExpect(element(by.id('biometric-prompt'))).toBeVisible();
    });
  });

  describe('Session Management', () => {
    it('should maintain session after app restart', async () => {
      // Login
      await element(by.id('login-button')).tap();
      await device.matchBiometric();

      // Restart app
      await device.terminateApp();
      await device.launchApp({ newInstance: false });

      // Should still be logged in (with biometric prompt)
      await device.matchBiometric();
      await detoxExpect(element(by.id('dashboard-screen'))).toBeVisible();
    });

    it('should revoke session on logout', async () => {
      await element(by.id('settings-tab')).tap();
      await element(by.id('logout-button')).tap();

      await detoxExpect(element(by.id('login-screen'))).toBeVisible();
    });

    it('should handle expired session', async () => {
      // This would require backend to expire the session
      // Simulated by clearing app state
      await device.clearKeychain();
      await device.reloadReactNative();

      await detoxExpect(element(by.id('login-screen'))).toBeVisible();
    });
  });

  afterAll(async () => {
    await device.terminateApp();
  });
});
