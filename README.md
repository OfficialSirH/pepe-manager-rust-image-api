# [Pepe Manager](https://pepemanager.com/)'s Image API Rust-Rewrite

## What is it?

This was a rewrite of the original image API in Rust for [Pepe Manager](https://pepemanager.com/), but due to complications, it was decided to undo this rewrite so I decided to make it open sourced(with what I was allowed to put open sourced).

## How do I use it?

set your env vars:

- `NODE_ENV` to `development` or `production`
- `IMAGE_API_PORT` to the port you want to use

change the allowed origins on line [20](https://github.com/OfficialSirH/pepe-manager-rust-image-api/blob/main/src/main.rs#L20) and [21](https://github.com/OfficialSirH/pepe-manager-rust-image-api/blob/main/src/main.rs#L21)

\*Keep in mind that the `image` parameter on the image manipulation functions is always assumed to be an image from Discord's CDN, so yes, it could easily break if you use other URLs that don't like the image file type being different when requested for(refer to line [43](https://github.com/OfficialSirH/pepe-manager-rust-image-api/blob/main/src/handlers.rs#L43) - [48](https://github.com/OfficialSirH/pepe-manager-rust-image-api/blob/main/src/handlers.rs#L48)).

## Where all of the images that Pepe Manager uses? I only see the enter and exit images.

I was given permission to just use only the enter and exit images, so I removed all of the other images and the code for manipulating them(but I kept it all in a private repo since it's still a lot of my work). You may notice a lot of the utilities are unused(cause rust-analyzer might scream about it), this is cause of the fact that the images that did use them aren't here.

## Credits

[CmdData](https://github.com/realCmdData) for the enter/exit images
