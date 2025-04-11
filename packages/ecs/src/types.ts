/* eslint-disable @typescript-eslint/no-explicit-any */

import type { Component } from './component';

export type ComponentTypeId = string;
/** Base type for all component constructors */
export interface ComponentConstructor<T extends Component = Component> {
  new (...args: any[]): T;
  typeId(): ComponentTypeId;
}

/** Type for any component constructor */
export type AnyComponentClass = ComponentConstructor<Component>;

/** Helper type to get component instances from constructor array */
export type ComponentInstances<T extends readonly ComponentConstructor[]> = {
  [K in keyof T]: T[K] extends ComponentConstructor<infer C> ? C : never;
};

/** Helper type to get component type from constructor */
export type ComponentInstanceType<T extends ComponentConstructor> =
  T extends ComponentConstructor<infer C> ? C : never;

export type Entity = number;
