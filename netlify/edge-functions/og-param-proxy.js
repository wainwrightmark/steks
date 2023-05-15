export default async (request, context) => {
    const url = new URL(request.url)

    // Get the page content.
    const response = await context.next()
    const page = await response.text()

    try{
        const game = url.pathname.substring(6);

        if (game.length < 4){
            return response;
        }

        const search = 'https://steks.net/icon/og_image.png'
        const replace = `https://steks.net/.netlify/functions/image?game=${game}`

        return new Response(page .replaceAll(search, replace), response);
    }
    catch{
        return response;
    }


}