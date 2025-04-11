import { describe, expect, test } from 'vitest';

import { Component } from '../component';
import { system } from '../system';
import { World } from '../world';

describe('System', () => {
  test('basic', () => {
    const world = new World();

    class Position extends Component {
      constructor(
        public x: number = 0,
        public y: number = 0,
      ) {
        super();
      }
    }

    class Velocity extends Component {
      constructor(
        public dx: number = 0,
        public dy: number = 0,
      ) {
        super();
      }
    }

    const movementSystem = system(
      (_, position, velocity) => {
        position.x += velocity.dx;
        position.y += velocity.dy;
      },
      [Position, Velocity],
    );

    world.spawn(new Position(0, 0), new Velocity(1, 2));

    const pos = new Position();
    const vel = new Velocity(1, 2);
    movementSystem.run(world, [pos, vel]);
    expect(pos.x).toBe(1);
    expect(pos.y).toBe(2);
  });
});
