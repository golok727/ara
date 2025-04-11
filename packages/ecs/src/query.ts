import type { AnyComponentClass, ComponentInstances } from './types';

export type Query<T extends AnyComponentClass[]> = (
  ...resolved: ComponentInstances<T>
) => void;
