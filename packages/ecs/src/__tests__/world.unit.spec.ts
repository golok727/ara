import { describe, expect, test, vi } from 'vitest';

import { Component } from '../component';
import { ComponentNotFoundError } from '../errors';
import { Scheduler } from '../schedule';
import { system } from '../system';
import { World } from '../world';

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

describe('world', () => {
  test('basic', () => {
    const world = new World();
    world.spawn(new Position(0, 0), new Velocity(1, 2));
    world.spawn(new Position(0, 0), new Velocity(1, 3));
    world.spawn(new Position(1, 0));

    const fn = vi.fn();
    world.query(fn, [Velocity, Position]);
    expect(fn).toHaveBeenCalledTimes(2);
  });

  test('schedule', () => {
    const world = new World();
    const scheduler = new Scheduler();

    const obj1 = {
      pos: new Position(0, 0),
      vel: new Velocity(1, 2),
    };
    const obj2 = {
      pos: new Position(0, 0),
      vel: new Velocity(1, 3),
    };
    const obj3 = {
      pos: new Position(1, 0),
    };

    world.spawn(obj1.pos, obj1.vel);
    world.spawn(obj2.pos, obj2.vel);

    const movementSystem = system(
      (_, position, velocity) => {
        position.x += velocity.dx;
        position.y += velocity.dy;
      },
      [Position, Velocity],
    );
    scheduler.addSystem(movementSystem);
    scheduler.run(world);
    expect(obj1.pos.x).toBe(1);
    expect(obj1.pos.y).toBe(2);
    expect(obj2.pos.x).toBe(1);
    expect(obj2.pos.y).toBe(3);
    expect(obj3.pos.x).toBe(1);
    expect(obj3.pos.y).toBe(0);
  });

  test('query should discard non matching layouts', () => {
    const world = new World();
    world.spawn(new Position(0, 0));
    world.spawn(new Position(0, 0), new Velocity(1, 2));
    world.spawn(new Velocity(1, 2));

    const fn = vi.fn();
    world.query(fn, [Position]);
    expect(fn).toHaveBeenCalledTimes(2);
  });

  test('thing', () => {
    const world = new World();
    const e1 = world.spawn(new Position(0, 0));
    const e2 = world.spawn(new Position(0, 0), new Velocity(1, 2));

    world.query(
      (position) => {
        position.x += 1;
      },
      [Position],
    );

    const e1Pos = world.view(e1).get(Position);
    const e2Pos = world.view(e2).get(Position);

    expect(e1Pos.x).toBe(1);
    expect(e2Pos.x).toBe(1);
  });

  describe('view', () => {
    test('get single component', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0));

      const view = world.view(entity);
      const position = view.get(Position);

      expect(position).toBeInstanceOf(Position);
      expect(position.x).toBe(0);
    });

    test('get with multiple components', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0), new Velocity(1, 2));

      const view = world.view(entity);
      const [position, velocity] = view.get(Position, Velocity);

      expect(position).toBeInstanceOf(Position);
      expect(position.x).toBe(0);
      expect(velocity).toBeInstanceOf(Velocity);
      expect(velocity.dx).toBe(1);
    });

    // todo will add function to make optional queries
    test.skip('getComponents optional should not throw and return undefined if not found', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0));
      const view = world.view(entity);
      const velocity = view.get(Velocity);
      expect(velocity).toBeUndefined();
    });

    test('get should throw if not found', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0));
      const view = world.view(entity);
      expect(() => view.get(Velocity)).toThrowError(ComponentNotFoundError);
    });

    test('get with multiple arguments should return array of components', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0), new Velocity(1, 2));

      const view = world.view(entity);
      const components = view.get(Position, Velocity);

      expect(components).toHaveLength(2);
      expect(components[0]).toBeInstanceOf(Position);
      expect(components[1]).toBeInstanceOf(Velocity);
    });

    test('get with multiple arguments should throw if not all components are present', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0));

      const view = world.view(entity);
      expect(() => view.get(Position, Velocity)).toThrowError(
        ComponentNotFoundError,
      );
    });

    test('has', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0));

      const view = world.view(entity);

      expect(view.has(Position)).toBe(true);
      expect(view.has(Velocity)).toBe(false);
    });

    test('getAll', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0), new Velocity(1, 2));

      const view = world.view(entity);
      const components = view.getAll();

      expect(components).toHaveLength(2);
      expect(components.get(Position.typeId())).toBeInstanceOf(Position);
      expect(components.get(Velocity.typeId())).toBeInstanceOf(Velocity);
    });

    test('map returned from getAll should be a copy', () => {
      const world = new World();
      const entity = world.spawn(new Position(0, 0), new Velocity(1, 2));

      const view = world.view(entity);
      const components = view.getAll();

      expect(components).toHaveLength(2);
      expect(components.get(Position.typeId())).toBeInstanceOf(Position);
      expect(components.get(Velocity.typeId())).toBeInstanceOf(Velocity);

      components.delete(Position.typeId());
      expect(view.get(Position)).toBeDefined();
    });
  });
});
