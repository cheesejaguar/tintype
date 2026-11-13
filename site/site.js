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
