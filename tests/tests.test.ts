import { meminfo, free } from "../src/lib";

describe("meminfo", () => {
    it("should get data from /proc/meminfo", () => {
        const result = meminfo();

        expect(typeof result).toBe("object");
    });
});

describe("free", () => {
    it("should get data from /proc/meminfo and the output format is like the \"free\" command ", () => {
        const result = free();

        expect(typeof result).toBe("object");
        expect(typeof result.mem).toBe("object");
        expect(typeof result.swap).toBe("object");
        expect(typeof result.mem.total).toBe("number");
        expect(typeof result.mem.used).toBe("number");
        expect(typeof result.mem.free).toBe("number");
        expect(typeof result.mem.shared).toBe("number");
        expect(typeof result.mem.buffers).toBe("number");
        expect(typeof result.mem.cache).toBe("number");
        expect(typeof result.mem.available).toBe("number");
        expect(typeof result.swap.total).toBe("number");
        expect(typeof result.swap.used).toBe("number");
        expect(typeof result.swap.free).toBe("number");
        expect(typeof result.swap.cache).toBe("number");
    });
});
