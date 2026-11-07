import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

test('fork update helper rejects installation while the feed is unavailable', () => {
  const script = new URL('../tv/self-update.sh', import.meta.url).pathname;
  for (const args of [
    [],
    ['/tmp/package.ipk', 'a'.repeat(64), '9.0.0', '/tmp/app', 'b'.repeat(32)],
  ]) {
    const result = spawnSync('/bin/sh', [script, ...args], { encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /updates are disabled/i);
  }
});
