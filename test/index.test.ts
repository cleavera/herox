import { strictEqual } from 'node:assert';
import { test } from 'node:test';
import { Keyboard, Mouse, Position, SpecialKey, unicode, Window } from '../index.js';

test('mouse move', () => {
  const mouse = new Mouse();
  mouse.moveTo(0, 0);
  const initial: Position = mouse.getPosition();
  strictEqual(initial.x, 0);
  strictEqual(initial.y, 0);

  mouse.moveTo(500, 600);
  const finalPosition: Position = mouse.getPosition();
  strictEqual(finalPosition.x, 500);
  strictEqual(finalPosition.y, 600);
});

test('key press', async () => {
  const keyboard = new Keyboard();

  keyboard.keyPress(unicode('a'));
  keyboard.keyPress(SpecialKey.Return);
});

test('screen capture', async () => {
  const window = Window.all().find(w => w.isFocused());
  const image = window!.captureImage();
  const target = { x: image.width / 2, y: image.height / 2 };

  const pixel = image.getPixelRgba(target.x, target.y);
  const matchingPixels = image.findRgbas(pixel);

  strictEqual(matchingPixels.some(p => p.x === target.x && p.y === target.y), true);
});
