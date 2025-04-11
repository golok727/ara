export class ComponentNotFoundError extends Error {
  constructor(entity: number, componentType: string) {
    super(`Component ${componentType} not found for entity ${entity}`);
    this.name = 'ComponentNotFoundError';
  }
}
