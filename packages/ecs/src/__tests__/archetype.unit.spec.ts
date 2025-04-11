import { describe, expect, test } from 'vitest';

import { EntityLayout } from '../archetype';
import { Component } from '../component';

class NameComponent extends Component {
  constructor(public name: string) {
    super();
  }
}

class PositionComponent extends Component {
  constructor(
    public x: number,
    public y: number,
  ) {
    super();
  }
}

class SpriteRendererComponent extends Component {
  constructor(public sprite: string) {
    super();
  }
}

describe('archetype', () => {
  describe('Entity Layout', () => {
    test('should work', () => {
      const layout = new EntityLayout([NameComponent, PositionComponent]);
      const layout2 = layout.clone();
      layout2.register(SpriteRendererComponent);

      expect(layout.componentCount).toBe(2);
      expect(layout2.componentCount).toBe(3);

      expect(layout).not.toBe(layout2);
      expect(layout2.hash()).not.toBe(layout.hash());
      expect(layout2.isCompatible(layout)).toBe(true);
      expect(layout.isCompatible(layout2)).toBe(false);
    });

    test('hasComponent works', () => {
      const layout = new EntityLayout([NameComponent, PositionComponent]);
      const layout2 = new EntityLayout([PositionComponent]);

      expect(layout.hasComponent(NameComponent)).toBe(true);
      expect(layout.hasComponent(PositionComponent)).toBe(true);
      expect(layout2.hasComponent(NameComponent)).toBe(false);
      expect(layout2.hasComponent(PositionComponent)).toBe(true);
    });

    test('should support multiple register methods', () => {
      class A extends Component {}
      class B extends Component {}
      class C extends Component {}

      const layout = new EntityLayout([NameComponent, PositionComponent]);
      layout.register(A, B).register(C);

      expect(layout.componentCount).toBe(5);
      expect(layout.hasComponent(NameComponent)).toBe(true);
      expect(layout.hasComponent(PositionComponent)).toBe(true);
      expect(layout.hasComponent(A)).toBe(true);
      expect(layout.hasComponent(B)).toBe(true);
      expect(layout.hasComponent(C)).toBe(true);
    });

    test('should clone layout', () => {
      const layout = new EntityLayout([NameComponent, PositionComponent]);
      const layout2 = layout.clone();

      expect(layout).not.toBe(layout2);
      expect(layout.hash()).toBe(layout2.hash());
      expect(layout.isCompatible(layout2)).toBe(true);
      expect(layout2.isCompatible(layout)).toBe(true);
      expect(layout.componentCount).toBe(2);
      expect(layout2.componentCount).toBe(2);
    });

    test('entity layout hash', () => {
      const layout = new EntityLayout([NameComponent, PositionComponent]);
      const layout2 = new EntityLayout([PositionComponent, NameComponent]);
      expect(layout.hash()).toBe(layout2.hash());
      expect(layout).not.toBe(layout2);
      expect(layout.isCompatible(layout2)).toBe(true);
      expect(layout2.isCompatible(layout)).toBe(true);
      expect(layout.componentCount).toBe(2);
      expect(layout2.componentCount).toBe(2);
    });

    test('unregister component', () => {
      const layout = new EntityLayout([
        NameComponent,
        PositionComponent,
        SpriteRendererComponent,
      ]);
      const oldHash = layout.hash();

      layout.unregister(NameComponent, SpriteRendererComponent);
      expect(layout.componentCount).toBe(1);
      expect(layout.hasComponent(NameComponent)).toBe(false);
      expect(layout.hasComponent(SpriteRendererComponent)).toBe(false);
      expect(layout.hasComponent(PositionComponent)).toBe(true);
      expect(layout.hash()).not.toBe(oldHash);
    });
  });
});
