import type { Component } from './component';
import type {
  Entity,
  AnyComponentClass,
  ComponentInstances,
  ComponentInstanceType,
  ComponentTypeId,
} from './types';

export interface IEntityView {
  entity: Entity;

  get<T extends AnyComponentClass>(componentType: T): ComponentInstanceType<T>;
  get<T extends AnyComponentClass[]>(
    ...componentTypes: T
  ): ComponentInstances<T>;

  getAll(): Map<ComponentTypeId, Component>;

  has(componentType: AnyComponentClass): boolean;
}

export interface IMutableEntityView extends IEntityView {
  /**
   * Batch operations to avoid multiple archetype updates.
   *
   * ```ts
   * view.addComponents(,...) // updates archetype
   *  view.removeComponents(...) // updates archetype
   *  view.batch(() => {
   *  view.addComponents(...);      // no  archetype update
   *  view.removeComponents(...);  // no archetype update
   *  view.removeComponent(...);  // no archetype update
   * })                          // archetype update
   *  view.removeAllComponents(); // archetype update

   * ```
   */
  batch(batchedCall: () => void): void;
  addComponents(...components: Component[]): void;
  removeComponents(...components: AnyComponentClass[]): void;
  removeAllComponents(): void;
}
