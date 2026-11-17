const status = document.getElementById('copy-status');
const resetTimers = new WeakMap();

function selectCommand(command) {
  const selection = window.getSelection();
  if (!selection) return;
  const range = document.createRange();
  range.selectNodeContents(command);
  selection.removeAllRanges();
  selection.addRange(range);
}

async function copyCommand(command) {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(command.textContent);
      return true;
    } catch {
      // Some browsers block the async clipboard on insecure or embedded pages.
    }
  }
  selectCommand(command);
  try {
    if (document.execCommand('copy')) {
      window.getSelection()?.removeAllRanges();
      return true;
    }
  } catch {
    // Preserve the selection so the user can copy it manually.
  }
  return false;
}

for (const button of document.querySelectorAll('button[data-copy]')) {
  const command = document.getElementById(button.dataset.copy);
  if (!command) continue;
  button.hidden = false;
  button.addEventListener('click', async () => {
    const copied = await copyCommand(command);
    status.textContent = '';
    const message = copied ? 'Command copied.' : 'Command selected. Use your keyboard or browser menu to copy.';
    requestAnimationFrame(() => { status.textContent = message; });
    button.textContent = copied ? 'Copied' : 'Selected';
    window.clearTimeout(resetTimers.get(button));
    resetTimers.set(button, window.setTimeout(() => {
      button.textContent = 'Copy';
    }, 2200));
  });
}

// Turn the stacked install commands into tabs, preselecting the visitor's system.
const install = document.getElementById('install');
const tablist = install?.querySelector('.install-tabs');
if (install && tablist) {
  const tabs = [...tablist.querySelectorAll('button')];
  const panels = tabs.map((tab) => document.getElementById(tab.getAttribute('aria-controls')));
  const select = (index, focus) => {
    tabs.forEach((tab, i) => {
      const selected = i === index;
      tab.setAttribute('aria-selected', String(selected));
      tab.tabIndex = selected ? 0 : -1;
      panels[i].hidden = !selected;
    });
    if (focus) tabs[index].focus();
  };
  tablist.setAttribute('role', 'tablist');
  tabs.forEach((tab, i) => {
    tab.setAttribute('role', 'tab');
    panels[i].setAttribute('role', 'tabpanel');
    panels[i].setAttribute('aria-labelledby', tab.id);
    tab.addEventListener('click', () => select(i, false));
    tab.addEventListener('keydown', (event) => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
      if (step) {
        event.preventDefault();
        select((i + step + tabs.length) % tabs.length, true);
      } else if (event.key === 'Home' || event.key === 'End') {
        event.preventDefault();
        select(event.key === 'Home' ? 0 : tabs.length - 1, true);
      }
    });
  });
  const platform = navigator.userAgentData?.platform || navigator.platform || '';
  select(/^win/i.test(platform) ? 1 : 0, false);
  install.classList.add('tabbed');
  tablist.hidden = false;
}

// The demo: one photo to the hosted endpoint, sent as the original file.
// The server enforces every limit; this only explains them.
const demo = document.getElementById('demo');
if (demo?.dataset.endpoint) {
  const endpoint = demo.dataset.endpoint;
  const maxBytes = Number(demo.dataset.maxBytes);
  const input = document.getElementById('demo-file');
  const demoStatus = document.getElementById('demo-status');
  const remaining = document.getElementById('demo-remaining');
  const result = document.getElementById('demo-result');
  const preview = document.getElementById('demo-preview');
  const verdict = document.getElementById('demo-verdict');
  const score = document.getElementById('demo-score');
  const explain = document.getElementById('demo-explain');
  const types = ['image/jpeg', 'image/png', 'image/webp'];
  let previewUrl = null;

  const say = (text, link) => {
    demoStatus.textContent = text;
    if (link) {
      demoStatus.append(' ');
      const a = document.createElement('a');
      a.href = '#install';
      a.textContent = link;
      demoStatus.append(a);
    }
  };
  const localTime = (seconds) => new Date(Date.now() + seconds * 1000)
    .toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  const minutes = (seconds) => {
    const m = Math.max(1, Math.ceil(seconds / 60));
    return m < 60 ? `${m} minute${m === 1 ? '' : 's'}` : `${Math.round(m / 6) / 10} hours`;
  };
  // `"ip-short";r=4;t=600, ...` into { 'ip-short': { r: 4, t: 600 }, ... }.
  const fields = (value) => {
    const out = {};
    for (const [, name, params] of (value || '').matchAll(/"([^"]+)"((?:;[a-z]+=\d+)*)/g)) {
      out[name] = Object.fromEntries([...params.matchAll(/;([a-z]+)=(\d+)/g)].map(([, k, v]) => [k, Number(v)]));
    }
    return out;
  };
  const showRemaining = (headers) => {
    const now = fields(headers.get('RateLimit'));
    const policy = fields(headers.get('RateLimit-Policy'));
    const short = now['ip-short'];
    const daily = now['ip-daily'];
    if (!short || !daily) return;
    const window = Math.round((policy['ip-short']?.w || 600) / 60);
    let text = `${short.r} of ${policy['ip-short']?.q ?? '?'} photos left in this ${window}-minute window · ${daily.r} of ${policy['ip-daily']?.q ?? '?'} left today.`;
    if (daily.r === 0) text += ` Your daily allowance starts again at midnight UTC (${localTime(daily.t)} your time).`;
    else if (short.r === 0) text += ` Your next photo can go in ${minutes(short.t)}, at ${localTime(short.t)}.`;
    remaining.textContent = text;
  };
  const finish = () => {
    input.disabled = false;
    input.value = '';
  };

  input.disabled = false;
  say('');
  input.addEventListener('change', async () => {
    const file = input.files?.[0];
    if (!file) return;
    result.hidden = true;
    if (file.size > maxBytes) {
      say(`This file is ${(file.size / 1e6).toFixed(1)} MB. The demo accepts photos up to ${maxBytes / 1e6} MB.`);
      finish();
      return;
    }
    if (file.type && !types.includes(file.type)) {
      say('The demo accepts JPEG, PNG or WebP photos only.');
      finish();
      return;
    }
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = URL.createObjectURL(file);
    preview.src = previewUrl;
    input.disabled = true;
    say('Scoring…');
    // The file goes as it is: no resizing or re-encoding, which would change
    // what the detector reads. Its name stays here.
    const body = new FormData();
    body.append('file', file, 'photo');
    let response;
    try {
      response = await fetch(endpoint, { method: 'POST', body, credentials: 'omit', cache: 'no-store', referrerPolicy: 'no-referrer' });
    } catch {
      say('The demo is busy or could not be reached. Try again in a minute, or', 'install Tintype to run it yourself.');
      finish();
      return;
    }
    showRemaining(response.headers);
    let data = null;
    try { data = await response.json(); } catch { /* not JSON */ }
    if (response.ok && typeof data?.score === 'number') {
      const flagged = data.verdict === 'ai-generated';
      verdict.textContent = flagged ? 'Flagged: likely AI-generated' : 'Not flagged: likely real';
      verdict.classList.toggle('flagged', flagged);
      score.textContent = `Score ${data.score.toFixed(3)} · photos above 0.500 are flagged`;
      explain.textContent = 'At this setting, Tintype flagged 1.2% of real photos and missed 10.3% of AI-generated ones in its blind benchmark. Resized, re-saved or screenshotted copies are harder to judge.';
      result.hidden = false;
      say('');
    } else if (response.status === 429 && data?.error?.limit === 'global_daily') {
      const wait = Number(response.headers.get('Retry-After')) || 0;
      say(`The demo is at capacity for today. It runs on a small hosting budget, so it scores a fixed number of photos per day across all visitors, and today's are used up. It starts again at midnight UTC (${localTime(wait)} your time). To keep going,`, 'install Tintype and run it on your own computer.');
    } else if (data?.error?.message) {
      say(data.error.message.charAt(0).toUpperCase() + data.error.message.slice(1).replace(/([^.])$/, '$1.'));
    } else {
      say(`The demo could not score this photo (HTTP ${response.status}). Try again later.`);
    }
    finish();
  });
}
