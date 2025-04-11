//https:github.com/shuding/stable-hash/blob/main/src/index.ts
export class StableHash {
  private static table = new WeakMap<object, string>();
  private static counter = 0;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static hash(arg: any): string {
    const type = typeof arg;
    const constructor = arg && arg.constructor;
    const isDate = constructor == Date;

    if (Object(arg) === arg && !isDate && constructor != RegExp) {
      // Object/function, not null/date/regexp. Use WeakMap to store the id first.
      // If it's already hashed, directly return the result.
      let result = this.table.get(arg);
      if (result) return result;
      // Store the hash first for circular reference detection before entering the
      // recursive `stableHash` calls.
      // For other objects like set and map, we use this id directly as the hash.
      result = ++this.counter + '~';
      this.table.set(arg, result);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      let index: any;

      if (constructor == Array) {
        // Array.
        result = '@';
        for (index = 0; index < arg.length; index++) {
          result += this.hash(arg[index]) + ',';
        }
        this.table.set(arg, result);
      } else if (constructor == Object) {
        // Object, sort keys.
        result = '#';
        const keys = Object.keys(arg).sort();
        while ((index = keys.pop() as string) !== undefined) {
          if (arg[index] !== undefined) {
            result += index + ':' + this.hash(arg[index]) + ',';
          }
        }
        this.table.set(arg, result);
      }
      return result;
    }
    if (isDate) return arg.toJSON();
    if (type == 'symbol') return arg.toString();
    return type == 'string' ? JSON.stringify(arg) : '' + arg;
  }
}
