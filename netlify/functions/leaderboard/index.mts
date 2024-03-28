import type { Context } from "@netlify/functions"
import { getStore } from "@netlify/blobs";


function set_headers(response : Response){
    response.headers.set('Access-Control-Allow-Origin', '*');
    response.headers.set('Access-Control-Allow-Methods', "GET, POST");
    response.headers.set('Access-Control-Allow-Headers', '*');
}

export default async (req: Request, context: Context) => {

    const url = new URL(req.url);
    const command = url.searchParams.get("command");
    const store = getStore({ name: "leaderboard", consistency: "strong" });


    switch (command?.toLowerCase()){
        case "get": {
            //we apparently don't ever use this

            const response = new Response("");

            set_headers(response);
            return response;

        }

        case "getrow":{
            const hash : string | null  = url.searchParams.get("hash");

            if (hash ){
                const data : string | null = await store.get(hash);

                if (data === null){
                    return new Response(`${hash} 0.0 0`);
                }

                const response = new Response(`${hash} ${data}`);

                set_headers(response);

                return response;
            }
            else{
                throw new Error("Could not get hash");
            }
        }

        case "set":{
            const hash : string | null  = url.searchParams.get("hash");
            const height : string  = url.searchParams.get("height")??"0.0";
            const height_f: number = parseFloat(height);
            const blob : string  = url.searchParams.get("blob")??"0";

            if (hash ){
                const data : string | null = await store.get(hash);

                if (data === null){

                    await store.set(hash, `${height} ${blob}`);
                    return new Response();
                }

                const split = data.split(" ");


                const prev_height = split[0];


                const prev_height_f : number = parseFloat(prev_height);

                if (height_f > prev_height_f){

                    await store.set(hash, `${height} ${blob}`);
                }

                const response = new Response();

                set_headers(response);

                return response;
            }
            else{
                throw new Error("Could not get hash");
            }
        }
        default:{
            throw new Error("Could not get command");
        }
    }
}
