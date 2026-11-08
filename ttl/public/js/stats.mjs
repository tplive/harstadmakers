import { getStoredMemberId } from './ttl-crypto.mjs';
import { fetchStatistics } from './unlock.mjs';
import {
  initializeLanguage,
  setLanguage,
  translate,
  translateDocument,
} from './i18n.mjs';

const status = document.getElementById('stats-status');
const dashboard = document.getElementById('stats-dashboard');
let currentStatistics = null;

function setStatus(message, kind = 'neutral') {
  status.textContent = message;
  status.className = `status-box ${kind}`;
}

function renderBars(elementId, values, formatter = (value) => value.label) {
  const element = document.getElementById(elementId);
  const maximum = Math.max(...values.map((value) => value.count), 1);
  element.replaceChildren(...values.map((value) => {
    const item = document.createElement('div');
    item.className = 'chart-bar-item';
    item.title = `${formatter(value)}: ${value.count}`;
    const fill = document.createElement('span');
    fill.className = 'chart-bar-fill';
    fill.style.height = `${Math.max((value.count / maximum) * 100, value.count ? 5 : 0)}%`;
    const label = document.createElement('small');
    label.textContent = formatter(value);
    const count = document.createElement('b');
    count.textContent = value.count;
    item.append(fill, count, label);
    return item;
  }));
}

function renderStatistics(statistics) {
  currentStatistics = statistics;
  document.getElementById('total-unlocks').textContent = statistics.successful_unlocks;
  document.getElementById('failed-attempts').textContent = statistics.failed_attempts;
  document.getElementById('active-months').textContent = statistics.monthly.filter((item) => item.count > 0).length;
  const busiest = statistics.by_hour.reduce((best, item) => item.count > best.count ? item : best, { label: '—', count: 0 });
  document.getElementById('busiest-hour').textContent = busiest.count ? `${busiest.label}:00` : '—';
  document.getElementById('busiest-hour-count').textContent = busiest.count
    ? `${busiest.count} ${translate('stats.unlocks')}`
    : translate('stats.noUnlocks');

  renderBars('monthly-chart', statistics.monthly);
  renderBars('hour-chart', statistics.by_hour, (item) => `${item.label}:00`);
  renderBars('weekday-chart', statistics.by_weekday, (item) => {
    const keys = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
    return translate(`weekday.${keys[Number(item.label)]}`) || item.label;
  });
  renderBars('yearly-chart', statistics.yearly);
}

async function loadStatistics() {
  const memberId = getStoredMemberId();
  if (!memberId) {
    setStatus(translate('stats.registerFirst'), 'error');
    return;
  }

  try {
    setStatus(translate('stats.loading'), 'neutral');
    const statistics = await fetchStatistics(memberId);
    renderStatistics(statistics);
    dashboard.classList.remove('hidden');
    status.classList.add('hidden');
  } catch (error) {
    setStatus(error.message || translate('stats.loadFailed'), 'error');
  }
}

const language = initializeLanguage();
translateDocument();
document.getElementById('language-select').value = language;
document.getElementById('language-select').addEventListener('change', (event) => {
  setLanguage(event.target.value);
  translateDocument();
  if (currentStatistics) {
    renderStatistics(currentStatistics);
  } else {
    loadStatistics();
  }
});
loadStatistics();
