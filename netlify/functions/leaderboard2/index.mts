import type { Context } from "@netlify/functions"
import { getStore } from "@netlify/blobs";


export default async (req: Request, context: Context) => {
    const store = getStore({ name: "animals", consistency: "strong" });

    await store.set("dog", "🐶");

    // This is a strongly-consistent read.
    const dog = await store.get("dog");

    return new Response(dog);
}