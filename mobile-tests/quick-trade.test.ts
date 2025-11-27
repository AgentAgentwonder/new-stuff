import { device, element, by, expect as detoxExpect, waitFor } from 'detox';

describe('Quick Trade', () => {
  beforeAll(async () => {
    await device.launchApp({
      newInstance: true,
      permissions: { notifications: 'YES' },
    });

    // Login first
    await element(by.id('login-button')).tap();
    await device.matchBiometric();
    await waitFor(element(by.id('dashboard-screen')))
      .toBeVisible()
      .withTimeout(5000);
  });

  describe('Trade Execution', () => {
    beforeEach(async () => {
      await element(by.id('quick-trade-tab')).tap();
    });

    it('should display quick trade interface', async () => {
      await detoxExpect(element(by.id('quick-trade-screen'))).toBeVisible();
      await detoxExpect(element(by.id('symbol-selector'))).toBeVisible();
      await detoxExpect(element(by.id('amount-input'))).toBeVisible();
      await detoxExpect(element(by.id('buy-button'))).toBeVisible();
      await detoxExpect(element(by.id('sell-button'))).toBeVisible();
    });

    it('should execute buy trade with biometric', async () => {
      await element(by.id('symbol-selector')).tap();
      await element(by.text('SOL')).tap();

      await element(by.id('amount-input')).clearText();
      await element(by.id('amount-input')).typeText('100');

      await element(by.id('buy-button')).tap();

      // Confirm trade details
      await detoxExpect(element(by.id('trade-confirmation'))).toBeVisible();
      await element(by.id('confirm-trade')).tap();

      // Biometric authentication
      await device.matchBiometric();

      // Wait for success
      await waitFor(element(by.id('trade-success')))
        .toBeVisible()
        .withTimeout(5000);
    });

    it('should execute sell trade with biometric', async () => {
      await element(by.id('symbol-selector')).tap();
      await element(by.text('SOL')).tap();

      await element(by.id('amount-input')).clearText();
      await element(by.id('amount-input')).typeText('50');

      await element(by.id('sell-button')).tap();

      // Confirm trade
      await element(by.id('confirm-trade')).tap();

      // Biometric authentication
      await device.matchBiometric();

      // Wait for success
      await waitFor(element(by.id('trade-success')))
        .toBeVisible()
        .withTimeout(5000);
    });

    it('should cancel trade on biometric failure', async () => {
      await element(by.id('symbol-selector')).tap();
      await element(by.text('USDC')).tap();

      await element(by.id('amount-input')).clearText();
      await element(by.id('amount-input')).typeText('200');

      await element(by.id('buy-button')).tap();
      await element(by.id('confirm-trade')).tap();

      // Fail biometric
      await device.unmatchBiometric();

      // Should show error
      await detoxExpect(element(by.id('trade-error'))).toBeVisible();
    });

    it('should respect trade amount limits', async () => {
      await element(by.id('symbol-selector')).tap();
      await element(by.text('SOL')).tap();

      // Try to exceed limit
      await element(by.id('amount-input')).clearText();
      await element(by.id('amount-input')).typeText('100000');

      await element(by.id('buy-button')).tap();

      // Should show limit error
      await detoxExpect(element(by.id('amount-limit-error'))).toBeVisible();
    });
  });

  describe('Safety Checks', () => {
    it('should display safety rules', async () => {
      await element(by.id('settings-tab')).tap();
      await element(by.id('safety-settings')).tap();

      await detoxExpect(element(by.id('safety-rules-list'))).toBeVisible();
      await detoxExpect(element(by.text(/Max notional value/i))).toBeVisible();
      await detoxExpect(element(by.text(/Max order size/i))).toBeVisible();
    });

    it('should require biometric for all trades', async () => {
      await element(by.id('quick-trade-tab')).tap();
      await element(by.id('buy-button')).tap();

      // Should always prompt for biometric
      await detoxExpect(element(by.id('biometric-prompt'))).toBeVisible();
    });
  });

  describe('Trade History', () => {
    it('should show executed trades in history', async () => {
      await element(by.id('history-tab')).tap();

      await detoxExpect(element(by.id('trade-history-list'))).toBeVisible();

      // Should have at least one trade from previous tests
      const firstTrade = element(by.id('trade-item-0'));
      await detoxExpect(firstTrade).toBeVisible();
    });

    it('should display trade details', async () => {
      await element(by.id('history-tab')).tap();
      await element(by.id('trade-item-0')).tap();

      await detoxExpect(element(by.id('trade-details'))).toBeVisible();
      await detoxExpect(element(by.id('trade-symbol'))).toBeVisible();
      await detoxExpect(element(by.id('trade-amount'))).toBeVisible();
      await detoxExpect(element(by.id('trade-price'))).toBeVisible();
      await detoxExpect(element(by.id('trade-timestamp'))).toBeVisible();
    });
  });

  afterAll(async () => {
    await device.terminateApp();
  });
});
