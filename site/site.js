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
