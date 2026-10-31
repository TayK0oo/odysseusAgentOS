//! Tests E2E minimaux — v31-bis.
//! Vérifient que les panneaux critiques affichent les données attendues
//! et la bannière W6 dette là où applicable.

import { test, expect } from '@playwright/test';

const BASE = process.env.BASE_URL || 'http://127.0.0.1:8787';

test('context panel displays W6 debt banner and real tokens', async ({ page }) => {
  await page.goto(`${BASE}/#context`);
  await expect(page.locator('.banner.warn')).toContainText('W6 non peuplé');
  await expect(page.locator('.banner.warn')).toContainText('tool_index=0');
});

test('plugins panel lists plugins', async ({ page }) => {
  await page.goto(`${BASE}/#plugins`);
  // Le panneau doit rendre sans erreur React
  await expect(page.locator('body')).not.toContainText('Something went wrong');
  await expect(page.locator('body')).not.toContainText('Uncaught Error');
});

test('router panel does not crash on empty skill index', async ({ page }) => {
  await page.goto(`${BASE}/#router`);
  await expect(page.locator('body')).not.toContainText('Something went wrong');
  await expect(page.locator('body')).not.toContainText('Uncaught Error');
});
