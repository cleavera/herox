import { strictEqual } from 'node:assert';
import { test } from 'node:test';
import { Herox, Position, SpecialKey, unicode } from '../index.js';

test('mouse move', () => {
  const herox = new Herox();
  herox.moveMouse(0, 0);
  const initial: Position = herox.getMousePosition();
  strictEqual(initial.x, 0);
  strictEqual(initial.y, 0);

  herox.moveMouse(500, 600);
  const finalPosition: Position = herox.getMousePosition();
  strictEqual(finalPosition.x, 500)
  strictEqual(finalPosition.y, 600)
});

test('key press', async () => {
  const herox = new Herox();

  herox.keyPress(unicode('a'));
  herox.keyPress(SpecialKey.Return);
});
