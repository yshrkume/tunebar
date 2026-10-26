import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const script = readFileSync(new URL('../src/scripts/media-bridge.js', import.meta.url), 'utf8');

function player() {
  let video = null;
  const polls = [];
  const messages = [];
  const window = {
    __TAURI_INTERNALS__: { invoke: async (...args) => { messages.push(args); } },
  };
  runInNewContext(script, {
    window,
    document: {
      readyState: 'complete', body: {},
      querySelector: selector => selector === 'video' ? video : selector === 'ytmusic-player-bar' ? {} : null,
    },
    MutationObserver: class { observe() {} disconnect() {} },
    setInterval: callback => polls.push(callback),
    console,
  });
  return {
    bridge: window.__MUSIC_BRIDGE__, messages,
    poll: () => polls.forEach(callback => callback()),
    attach() {
      video = { paused: true, plays: 0, pauses: 0,
        play() { this.paused = false; this.plays++; return Promise.resolve(); },
        pause() { this.paused = true; this.pauses++; },
        addEventListener() {},
      };
      return video;
    },
  };
}

test('a command received before video creation executes only once', () => {
  const p = player();
  p.bridge.runCommand('toggle');
  const video = p.attach();
  p.poll();
  p.poll();
  assert.equal(video.plays, 1);
  assert.equal(video.pauses, 0);
});

test('queued commands preserve order and invalid commands never execute', () => {
  const p = player();
  assert.equal(p.bridge.runCommand('arbitrary()'), false);
  p.bridge.runCommand('play');
  p.bridge.runCommand('pause');
  const video = p.attach();
  p.poll();
  assert.equal(video.plays, 1);
  assert.equal(video.pauses, 1);
  assert.equal(video.paused, true);
});

test('metadata and playback notifications are deduplicated', () => {
  const p = player();
  p.attach();
  p.poll();
  p.poll();
  assert.equal(p.messages.filter(([name]) => name === 'update_playback_state').length, 1);
  p.bridge.runCommand('play');
  p.poll();
  assert.equal(p.messages.filter(([name]) => name === 'update_playback_state').length, 2);
});

test('offline command queue is bounded', () => {
  const p = player();
  for (let i = 0; i < 100; i++) p.bridge.runCommand('play');
  const video = p.attach();
  p.poll();
  assert.equal(video.plays, 32);
});
