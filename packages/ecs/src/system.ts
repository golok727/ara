import type { AnyComponentClass, ComponentInstances } from './types';

import { World } from './world';

export interface SystemConfig<
  T extends AnyComponentClass[] = AnyComponentClass[],
> {
  readonly query: T;
  run: SystemRunner<T>;
}

export type SystemRunner<T extends AnyComponentClass[]> = (
  world: World,
  components: ComponentInstances<T>,
) => void;

export function system<const T extends AnyComponentClass[] = []>(
  runner: (world: World, ...components: ComponentInstances<T>) => void,
  query: T,
): SystemConfig<T> {
  return {
    query: query,
    run: (world, components) => {
      runner(world, ...(components as ComponentInstances<T>));
    },
  };
}
