import { strictEqual } from 'node:assert';
import { test } from 'node:test';
import { Keyboard, Mouse, Position, SpecialKey, unicode, Window } from '../index.js';

test('mouse move', async () => {
  const mouse = new Mouse();
  await mouse.moveTo(0, 0);
  const initial: Position = await mouse.getPosition();
  strictEqual(initial.x, 0);
  strictEqual(initial.y, 0);

  await mouse.moveTo(500, 600);
  const finalPosition: Position = await mouse.getPosition();
  strictEqual(finalPosition.x, 500);
  strictEqual(finalPosition.y, 600);
});

test('humanlike mouse move', async () => {
  const mouse = new Mouse();
  await mouse.humanlikeMoveTo(0, 0, 1000);
  const initial: Position = await mouse.getPosition();
  strictEqual(initial.x, 0);
  strictEqual(initial.y, 0);

  await mouse.humanlikeMoveTo(500, 600, 1000);
  const finalPosition: Position = await mouse.getPosition();
  strictEqual(finalPosition.x, 500);
  strictEqual(finalPosition.y, 600);
});

test('multiple mouse tasks done simultaneously', async() => {
  const mouse = new Mouse();

  const results: Array<Error> = (await Promise.all([
    mouse.humanlikeMoveTo(0, 0, 100).then(() => null, e => e),
    mouse.humanlikeMoveTo(500, 600, 100).then(() => null, e => e),
  ])).filter(e => e !== null);

  strictEqual(results.length, 1);
  strictEqual(results[0] instanceof Error, true);
});

test('key press', async () => {
  const keyboard = new Keyboard();

  keyboard.keyPress(unicode('a'));
  keyboard.keyPress(SpecialKey.Backspace);
});

test('screen capture', async () => {
  const window = Window.all().find(w => w.isFocused());
  const image = await window!.captureImage();
  const target = { x: Math.ceil(image.width / 2), y: Math.ceil(image.height / 2) };

  const pixel = await image.getPixelRgba(target.x, target.y);
  const matchingPixels = await image.findRgbas(pixel);

  strictEqual(matchingPixels.some(p => p.x === target.x && p.y === target.y), true);
});

