import type { Query } from './query';
import type {
  AnyComponentClass,
  ComponentInstances,
  ComponentInstanceType,
  ComponentTypeId,
  Entity,
} from './types';
import type { IEntityView } from './view';

import { Archetypes, EntityLayout } from './archetype';
import { ComponentStore, type Component } from './component';
import { ComponentNotFoundError } from './errors';

export class World {
  private _nextEntity: Entity = 1;
  private _archetypes = new Archetypes();
  private _componentStore: ComponentStore = new ComponentStore();

  /**
   * Executes a query on entities that match the specified component types and returns the list of matching entities.
   *
   * @template T - A tuple of component classes that define the query's component types.
   * @param query - A callback function that is invoked for each matching entity. It receives the components of the entity as arguments.
   * @param componentTypes - An array of component classes that define the required components for the query.
   * @returns An array of entities that match the specified component types.
   *
   * @remarks
   * This method filters archetypes based on their compatibility with the provided component types.
   * For each matching archetype, it retrieves the entities and their corresponding components,
   * then invokes the query callback with the components as arguments.
   *
   * The method ensures type safety by leveraging the compatibility of archetypes with the provided component types.
   */
  query<const T extends AnyComponentClass[]>(
    query: Query<T>,
    componentTypes: T,
  ) {
    const layout = new EntityLayout(componentTypes);

    const archeTypes = this._archetypes
      .all()
      .filter((a) => a.layout.isCompatible(layout));

    const result = [];

    for (const archetype of archeTypes) {
      const entities = archetype.getEntities();

      for (const entity of entities) {
        const components = componentTypes.map((cstr) =>
          this._componentStore.get(entity, cstr),
        ) as Component[]; // Safety: we know it would not be undefined thanks archetype;
        query(...(components as never));
      }

      result.push(...entities);
    }

    return result;
  }

  /**
   * Creates a view for the specified entity, allowing access to its components and interactions
   * within the world.
   *
   * @param entity - The entity for which the view is to be created.
   * @returns An instance of `View` representing the specified entity.
   */
  view(entity: Entity): IEntityView {
    return new EntityView(entity, this._componentStore);
  }

  /**
   * Spawns a new entity in the world and associates it with the provided components.
   *
   * @param components - A variable number of components to associate with the new entity.
   *                      Each component is an instance of a class that represents a specific
   *                      aspect of the entity's behavior or data.
   * @returns The unique identifier of the newly created entity.
   *
   * @remarks
   * - If no components are provided, the entity is created without any associated components.
   * - The method calculates a unique layout hash for the combination of components and
   *   ensures that the entity is added to the appropriate archetype.
   * - Components are stored in the internal component store for efficient access.
   */
  spawn(...components: Component[]): Entity {
    const entity = this._nextEntity++;

    if (components.length <= 0) {
      return entity;
    }

    const layout = new EntityLayout(
      components.map(
        (component) => component.constructor,
      ) as AnyComponentClass[],
    );

    const archetype = this._archetypes.getOrInsert(layout);

    archetype.add(entity);

    for (const component of components) {
      this._componentStore.add(entity, component);
    }

    return entity;
  }
}

class EntityView implements IEntityView {
  constructor(
    public entity: Entity,
    protected store: ComponentStore,
  ) {}

  get<T extends AnyComponentClass>(componentType: T): ComponentInstanceType<T>;
  get<T extends AnyComponentClass[]>(
    ...componentTypes: T
  ): ComponentInstances<T>;
  get<T extends AnyComponentClass | AnyComponentClass[]>(
    ...componentTypes: T extends AnyComponentClass[] ? T : [T]
  ): T extends AnyComponentClass[]
    ? ComponentInstances<T>
    : T extends AnyComponentClass
      ? ComponentInstanceType<T>
      : never {
    if (componentTypes.length === 0) {
      throw new Error('At least one component type is required');
    }

    const components = [];
    for (const componentType of componentTypes) {
      const component = this.store.get(this.entity, componentType);
      if (!component) {
        throw new ComponentNotFoundError(this.entity, componentType.name);
      }
      components.push(component);
    }

    if (componentTypes.length === 1) {
      return components[0] as never;
    }

    return components as never;
  }

  getAll(): Map<ComponentTypeId, Component> {
    return this.store.getAll(this.entity);
  }

  has(componentType: AnyComponentClass): boolean {
    const component = this.store.get(this.entity, componentType);
    return !!component;
  }
}
