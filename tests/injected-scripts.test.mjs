import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const read = name => readFileSync(new URL(`../src/scripts/${name}.js`, import.meta.url), 'utf8');

test('ad skipping does not seek to infinity or change playback rate', () => {
  let skip;
  let sought = false;
  const video = { duration: Infinity, playbackRate: 1, set currentTime(_) { sought = true; } };
  runInNewContext(read('ad-blocker'), {
    document: {
      readyState: 'complete', body: {},
      createElement: () => ({}), documentElement: { appendChild() {} },
      querySelector: selector => selector === 'video' ? video : selector === '.ad-showing' ? {} : null,
    },
    setInterval: callback => { skip = callback; },
    MutationObserver: class { observe() {} },
  });
  skip();
  assert.equal(sought, false);
  assert.equal(video.playbackRate, 1);
});

test('slow initialization is not falsely covered with a network error', () => {
  let overlays = 0;
  const listeners = {};
  runInNewContext(read('error-handler'), {
    document: {
      readyState: 'complete',
      getElementById: () => null,
      documentElement: { appendChild() { overlays++; } },
    },
    navigator: { onLine: true },
    window: { addEventListener: (name, fn) => { listeners[name] = fn; } },
    // Executing scheduled checks immediately also catches the former 5s heuristic.
    setTimeout: fn => fn(), console,
  });
  assert.equal(overlays, 0);
  assert.equal(typeof listeners.offline, 'function');
});
