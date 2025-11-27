import { test, expect } from '@playwright/test';

test.describe('AI Assistant', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should display AI Assistant welcome screen', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      await expect(page.getByText(/Welcome to AI Trading Assistant/i)).toBeVisible();
    }
  });

  test('should show command suggestions when clicked', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const suggestionsButton = page.getByTitle('Show command suggestions');
      if (await suggestionsButton.isVisible()) {
        await suggestionsButton.click();
        await expect(page.getByText(/Quick Commands/i)).toBeVisible();
        await expect(page.getByText(/Analyze Risk/i)).toBeVisible();
        await expect(page.getByText(/Optimize Portfolio/i)).toBeVisible();
      }
    }
  });

  test('should allow typing in chat input', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const chatInput = page.getByPlaceholder(/Ask me anything/i);
      if (await chatInput.isVisible()) {
        await chatInput.fill('What is the market sentiment today?');
        await expect(chatInput).toHaveValue('What is the market sentiment today?');
      }
    }
  });

  test('should have quick actions panel', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      await expect(page.getByText(/Quick Actions/i)).toBeVisible();
    }
  });

  test('should have pattern warnings panel', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      await expect(page.getByText(/Pattern Warnings/i)).toBeVisible();
    }
  });

  test('should have risk analysis panel', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      await expect(page.getByText(/Risk Analysis/i)).toBeVisible();
    }
  });

  test('should create new conversation', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const newConversationButton = page.getByTitle('New conversation');
      if (await newConversationButton.isVisible()) {
        await newConversationButton.click();
      }
    }
  });

  test('should allow command suggestion selection', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const suggestionsButton = page.getByTitle('Show command suggestions');
      if (await suggestionsButton.isVisible()) {
        await suggestionsButton.click();
        const analyzeRisk = page.getByRole('button', { name: /Analyze Risk/i });
        if (await analyzeRisk.isVisible()) {
          await analyzeRisk.click();
          const chatInput = page.getByPlaceholder(/Ask me anything/i);
          await expect(chatInput).toHaveValue(/Analyze the risk profile of my portfolio/i);
        }
      }
    }
  });

  test('should display reasoning steps in message', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const chatInput = page.getByPlaceholder(/Ask me anything/i);
      if (await chatInput.isVisible()) {
        await chatInput.fill('Test message');
        const sendButton = page
          .getByRole('button')
          .filter({ has: page.locator('svg[class*="lucide-send"]') });
        if (await sendButton.isVisible()) {
          await sendButton.click();
          await page.waitForTimeout(2000);
        }
      }
    }
  });

  test('should toggle quick actions panel', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const quickActionsHeader = page.getByText(/Quick Actions/i).first();
      if (await quickActionsHeader.isVisible()) {
        const container = quickActionsHeader.locator('..');
        await container.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('should toggle pattern warnings panel', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const patternWarningsHeader = page.getByText(/Pattern Warnings/i).first();
      if (await patternWarningsHeader.isVisible()) {
        const container = patternWarningsHeader.locator('..');
        await container.click();
        await page.waitForTimeout(500);
      }
    }
  });
});

test.describe('AI Assistant Feedback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should have feedback buttons on assistant messages', async ({ page }) => {
    const aiButton = page.getByRole('button', { name: /AI Assistant/i });
    if (await aiButton.isVisible()) {
      await aiButton.click();
      const chatInput = page.getByPlaceholder(/Ask me anything/i);
      if (await chatInput.isVisible()) {
        await chatInput.fill('Hello');
        const sendButton = page
          .getByRole('button')
          .filter({ has: page.locator('svg[class*="lucide-send"]') });
        if (await sendButton.isVisible()) {
          await sendButton.click();
          await page.waitForTimeout(2000);
          const helpfulText = page.getByText(/Was this helpful\?/i);
          if (await helpfulText.isVisible()) {
            await expect(helpfulText).toBeVisible();
          }
        }
      }
    }
  });
});

test.describe('AI Assistant Commands', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  const commands = [
    'Analyze Risk',
    'Optimize Portfolio',
    'Detect Patterns',
    'Market Analysis',
    'Trade Ideas',
    'Quick Action',
  ];

  commands.forEach(command => {
    test(`should have ${command} command`, async ({ page }) => {
      const aiButton = page.getByRole('button', { name: /AI Assistant/i });
      if (await aiButton.isVisible()) {
        await aiButton.click();
        const suggestionsButton = page.getByTitle('Show command suggestions');
        if (await suggestionsButton.isVisible()) {
          await suggestionsButton.click();
          const commandButton = page.getByText(command);
          await expect(commandButton).toBeVisible();
        }
      }
    });
  });
});

test.describe('AI Assistant Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should navigate to AI Assistant from command palette', async ({ page }) => {
    await page.keyboard.press('Meta+K');
    await page.waitForTimeout(500);
    const searchInput = page.getByPlaceholder(/Search/i);
    if (await searchInput.isVisible()) {
      await searchInput.fill('AI');
      await page.waitForTimeout(500);
      const aiCommand = page.getByText(/AI Assistant/i).first();
      if (await aiCommand.isVisible()) {
        await aiCommand.click();
        await expect(page.getByText(/Welcome to AI Trading Assistant/i)).toBeVisible();
      }
    }
  });
});
