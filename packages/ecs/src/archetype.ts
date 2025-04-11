import type { AnyComponentClass, ComponentTypeId } from './types';
import type { Entity } from './types';

// layout for an entity
export type ArchetypeId = string;
export class EntityLayout {
  private static componentBitPositions: Map<
    ComponentTypeId,
    { arrayIndex: number; bitPos: number }
  > = new Map();
  private static nextBitPosition: number = 0;
  private _components: Uint32Array; // Array of 32-bit integers
  private _hash?: string;
  private _count: number = 0;

  constructor(componentTypes?: AnyComponentClass[]) {
    // Assume max 128 components (4 * 32 bits) for this example; adjust as needed
    this._components = new Uint32Array(4);
    this.register(...(componentTypes || []));
  }

  /**
   *   Check if this layout contains the component type
   *  @param componentType - The component type to check
   *  @returns true if the layout contains the component type, false otherwise
   * */
  hasComponent(componentType: AnyComponentClass): boolean {
    const id = componentType.typeId();
    const pos = EntityLayout.componentBitPositions.get(id);
    if (pos === undefined) {
      return false;
    }
    return (this._components[pos.arrayIndex] & (1 << pos.bitPos)) !== 0;
  }

  /**
   *   Get the number of components in this layout
   *  @returns The number of components in this layout
   * */
  get componentCount(): number {
    return this._count;
  }

  /**
   *
   * @returns A new EntityLayout instance that is a clone of this one
   */
  clone() {
    const clone = new EntityLayout();
    clone._components.set(this._components);
    clone._hash = this._hash;
    clone._count = this._count;
    return clone;
  }

  /**
   *   Register a component type with this layout
   *  @param componentTypes - The component types to register
   *  @returns this for method chaining
   * */
  register(...componentTypes: AnyComponentClass[]) {
    if (componentTypes.length === 0) {
      return this;
    }

    for (const cstr of componentTypes) {
      const id = cstr.typeId();
      let pos = EntityLayout.componentBitPositions.get(id);
      if (pos === undefined) {
        const arrayIndex = Math.floor(EntityLayout.nextBitPosition / 32);
        const bitPos = EntityLayout.nextBitPosition % 32;
        pos = { arrayIndex, bitPos };
        EntityLayout.componentBitPositions.set(id, pos);
        EntityLayout.nextBitPosition++;
        if (arrayIndex >= this._components.length) {
          // Resize array if needed (rare case)
          const newArray = new Uint32Array(arrayIndex + 1);
          newArray.set(this._components);
          this._components = newArray;
        }
      }
      this._components[pos.arrayIndex] |= 1 << pos.bitPos; // Set the bit
      this._count++;
    }

    this._hash = undefined;
    return this;
  }

  /**
   *  Unregister a component type from this layout
   *  @param componentTypes - The component types to unregister
   *  @returns this for method chaining
   * */
  unregister(...componentTypes: AnyComponentClass[]) {
    if (componentTypes.length === 0) {
      return this;
    }
    for (const cstr of componentTypes) {
      const id = cstr.typeId();
      const pos = EntityLayout.componentBitPositions.get(id);
      if (pos !== undefined) {
        this._components[pos.arrayIndex] &= ~(1 << pos.bitPos); // Clear the bit
        this._count--;
        EntityLayout.componentBitPositions.delete(id);
      }
    }
    this._hash = undefined;
    return this;
  }

  isEmpty(): boolean {
    return this._count <= 0;
  }

  /**
   * get a string hash of the layout
   */
  hash() {
    if (!this._hash) {
      this._hash = this._components.join('|'); // Simple join of array elements
    }
    return this._hash;
  }

  /**
   *   Check if this layout is the same as the other layout
   *  @param other - The other layout to compare with
   *  @returns true if the layouts are the same, false otherwise
   * */
  is(other: EntityLayout): boolean {
    return this._hash === other.hash();
  }

  /**
   *   Check if this layout contains all components of the other layout
   */
  isCompatible(other: EntityLayout): boolean {
    if (this._count < other._count) {
      return false;
    }

    for (let i = 0; i < other._components.length; i++) {
      if (
        (this._components[i] & other._components[i]) !==
        other._components[i]
      ) {
        return false;
      }
    }
    return true;
  }
}

/**
 *  Archetype is a collection of entities that share the same component layout
 */
export class Archetype {
  private entities: Set<Entity> = new Set();

  constructor(
    public readonly id: ArchetypeId,
    public readonly layout: EntityLayout,
  ) {}

  add(entity: Entity) {
    this.entities.add(entity);
  }

  remove(entity: Entity) {
    this.entities.delete(entity);
  }

  has(entity: Entity): boolean {
    return this.entities.has(entity);
  }

  getEntities(): Entity[] {
    return [...this.entities];
  }
}

export class Archetypes {
  private _store: Map<ArchetypeId, Archetype> = new Map();

  all() {
    return [...this._store.values()];
  }

  updateLayout(
    entity: Entity,
    oldLayout: EntityLayout,
    newLayout: EntityLayout,
  ): Archetype | undefined {
    const archetype = this._store.get(oldLayout.hash());

    if (!archetype) {
      throw new Error(`Archetype ${oldLayout.hash()} not found`);
    }

    if (!archetype.has(entity)) {
      throw new Error(
        `Entity ${entity} does not belong in archetype ${oldLayout.hash()}`,
      );
    }

    if (oldLayout.is(newLayout)) {
      return;
    }

    archetype.remove(entity);

    const newArchetype = this.getOrInsert(newLayout);
    newArchetype.add(entity);
    return newArchetype;
  }

  get(idOrLayout: ArchetypeId | EntityLayout): Archetype | undefined {
    const id = 'string' === typeof idOrLayout ? idOrLayout : idOrLayout.hash();
    return this._store.get(id);
  }

  getOrInsert(layout: EntityLayout): Archetype {
    const id = layout.hash();
    let archetype = this._store.get(id);
    if (!archetype) {
      archetype = new Archetype(id, layout);
      this._store.set(id, archetype);
    }
    return archetype;
  }
}
