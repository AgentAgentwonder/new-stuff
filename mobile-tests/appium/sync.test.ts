import { remote, Browser } from 'webdriverio';

let driver: Browser<'async'>;

describe('Mobile Sync', () => {
  beforeAll(async () => {
    driver = await remote({
      logLevel: 'error',
      path: '/wd/hub',
      capabilities: {
        platformName: process.env.APPIUM_PLATFORM ?? 'iOS',
        'appium:deviceName': process.env.APPIUM_DEVICE ?? 'iPhone 14',
        'appium:app': process.env.APPIUM_APP ?? '/path/to/EclipseMobile.app',
        'appium:automationName': process.env.APPIUM_AUTOMATION ?? 'XCUITest',
        'appium:platformVersion': process.env.APPIUM_PLATFORM_VERSION ?? '17.0',
      },
    });
  }, 60000);

  it('should sync portfolio data', async () => {
    const syncButton = await driver.$('~sync-button');
    await syncButton.waitForExist({ timeout: 5000 });
    await syncButton.click();

    const status = await driver.$('~sync-status');
    await status.waitForExist({ timeout: 5000 });

    const text = await status.getText();
    expect(text.toLowerCase()).toContain('synced');
  });

  it('should receive push notification payload', async () => {
    const notificationsTab = await driver.$('~notifications-tab');
    await notificationsTab.click();

    const notificationItem = await driver.$('~notification-item-0');
    await notificationItem.waitForExist({ timeout: 5000 });

    const title = await notificationItem.$('~notification-title');
    const body = await notificationItem.$('~notification-body');

    expect(await title.getText()).toBeTruthy();
    expect(await body.getText()).toBeTruthy();
  });

  afterAll(async () => {
    if (driver) {
      await driver.deleteSession();
    }
  });
});
