interface Mem {
    total: number;
    used: number;
    free: number;
    shared: number;
    buff: number;
    cache: number;
    available: number;
}

interface Swap {
    total: number;
    used: number;
    free: number;
}

interface Free {
    mem: Mem;
    swap: Swap;
}

declare module "node-meminfo" {
    /**
     * Get memory information. The unit is byte.
     */
    export function get(): object;

    /**
     * Get memory information to "free" style. The unit is byte.
     */
    export function free(): Free;
}
