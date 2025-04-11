import type { SystemConfig } from './system';
import type { AnyComponentClass } from './types';
import type { World } from './world';

export class Scheduler {
  systems: SystemConfig[] = [];

  addSystem<T extends AnyComponentClass[] = AnyComponentClass[]>(
    system: SystemConfig<T>,
  ) {
    if (this.systems.includes(system as never)) {
      throw new Error('System already added');
    }

    this.systems.push(system as never);
  }

  run(world: World) {
    for (const system of this.systems) {
      world.query(
        (...components) => system.run(world, components),
        system.query,
      );
    }
  }
}
