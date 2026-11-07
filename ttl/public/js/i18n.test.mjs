import test from 'node:test';
import assert from 'node:assert/strict';
import {
  getLanguage,
  initializeLanguage,
  setLanguage,
  SUPPORTED_LANGUAGES,
  translate,
  translations,
} from './i18n.mjs';

const values = new Map();
globalThis.localStorage = {
  getItem: (key) => values.get(key) ?? null,
  setItem: (key, value) => values.set(key, String(value)),
  removeItem: (key) => values.delete(key),
};

test('supports the initial languages', () => {
  assert.deepEqual(Object.keys(SUPPORTED_LANGUAGES), ['en', 'nb_no', 'se']);
});

test('uses English when a translation is missing', () => {
  setLanguage('se');
  assert.equal(translate('status.checking'), 'Checking registration status...');
});

test('changes and persists the selected language', () => {
  setLanguage('nb_no');
  assert.equal(getLanguage(), 'nb_no');
  assert.equal(translate('hero.heading'), 'Dørtilgang');

  setLanguage('en');
  assert.equal(initializeLanguage(), 'en');
  assert.equal(getLanguage(), 'en');
});

test('stores translations by key with per-language entries', () => {
  assert.deepEqual(translations['hero.heading'], {
    en: 'Door access',
    nb_no: 'Dørtilgang',
    se: 'Uksaláhtta',
  });
  assert.equal(translations['status.checking'].se, undefined);
});

test('falls back to English for an unsupported language', () => {
  assert.equal(setLanguage('de'), 'en');
  assert.equal(translate('hero.heading'), 'Door access');
});
