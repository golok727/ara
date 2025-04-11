import type {
  AnyComponentClass,
  ComponentInstanceType,
  ComponentTypeId,
  Entity,
} from './types';

import { StableHash } from './utils';

const ComponentBrand = Symbol('ara/ecs:component');

// marker to identify component classes
export abstract class Component {
  private static _typeIdMap = new WeakMap<typeof Component, ComponentTypeId>();

  static typeId(): ComponentTypeId {
    const cache = this._typeIdMap.get(this);
    if (cache) {
      return cache;
    }
    const typeId = `${this.name}:${StableHash.hash(this)}`;
    this._typeIdMap.set(this, typeId);
    return typeId;
  }

  [ComponentBrand] = true;
}

export class ComponentStore {
  components: Map<Entity, Map<ComponentTypeId, Component>> = new Map();

  has(entity: Entity, type: AnyComponentClass): boolean {
    const components = this.components.get(entity);
    if (!components) {
      return false;
    }
    const typeId = type.typeId();
    return components.has(typeId);
  }

  getAll(entity: Entity): Map<ComponentTypeId, Component> {
    const components = this.components.get(entity);
    if (!components) {
      return new Map();
    }
    return new Map(components);
  }

  add(entity: Entity, component: Component) {
    const cstr = component.constructor as typeof Component;
    const typeId = cstr.typeId();

    if (!this.components.has(entity)) {
      this.components.set(entity, new Map());
    }
    this.components.get(entity)!.set(typeId, component);
  }

  get<T extends AnyComponentClass>(
    entity: Entity,
    type: AnyComponentClass,
  ): ComponentInstanceType<T> | undefined {
    return this.components.get(entity)?.get(type.typeId()) as never;
  }

  remove(entity: Entity, type: AnyComponentClass) {
    const components = this.components.get(entity);
    if (!components) {
      return;
    }
    const typeId = type.typeId();
    components.delete(typeId);
  }

  clear(entity: Entity) {
    this.components.delete(entity);
  }
}
