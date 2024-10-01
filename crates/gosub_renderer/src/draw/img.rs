use std::io::Cursor;

use anyhow::anyhow;

use gosub_net::http::fetcher::Fetcher;
use gosub_net::http::response::Response;
use gosub_render_backend::svg::SvgRenderer;
use gosub_render_backend::{Image as _, ImageBuffer, RenderBackend, SizeU32};
use gosub_shared::types::Result;

pub fn request_img<B: RenderBackend>(
    fetcher: &Fetcher,
    svg_renderer: &mut B::SVGRenderer,
    url: &str,
    size: Option<SizeU32>,
) -> Result<ImageBuffer<B>> {
    
    
    return Ok(ImageBuffer::Image(B::Image::from_img(image::DynamicImage::new_rgba8(1, 1))));
    
    
    let res: Response = todo!();
    
    // let res = fetcher.get(url)?;

    if !res.is_ok() {
        return Err(anyhow!("Could not get url. Status code {}", res.status));
    }

    let img = res.body;

    let is_svg = img.starts_with(b"<?xml") || img.starts_with(b"<svg");

    Ok(if is_svg {
        let svg = String::from_utf8(img)?; //TODO: We need to handle non-utf8 SVGs here

        let svg = <B::SVGRenderer as SvgRenderer<B>>::parse_external(svg)?;

        if let Some(size) = size {
            svg_renderer.render_with_size(&svg, size)?
        } else {
            svg_renderer.render(&svg)?
        }
    } else {
        let format = image::guess_format(&img)?;
        let img = image::load(Cursor::new(img), format)?; //In that way we don't need to copy the image data

        let img = B::Image::from_img(img);

        ImageBuffer::Image(img)
    })
}
