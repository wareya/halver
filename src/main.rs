use std::env;

#[allow(unused_mut)]

use image::GenericImageView;
use image::Pixel;
use core::cmp;
use std::cell::RefCell;
use std::convert::TryInto;
use ordered_float::OrderedFloat;

fn main()
{
    let mut in_filename = String::new();
    let mut out_filename = String::new();
    let mut in_size = 32;
    let mut out_size = 16;
    
    for arg in env::args().skip(1)
    {
        if in_filename.len() == 0
        {
            in_filename = arg.clone();
            continue;
        }
        if out_filename.len() == 0
        {
            out_filename = arg.clone();
            continue;
        }
    }
    if out_filename == ""
    {
        println!(
"usage:
halver <infile> <outfile>
");
        return;
    }
    
    let in_img = RefCell::new(image::open(in_filename).unwrap().to_rgba8());
    let in_dim = in_img.borrow().dimensions();
    let out_dim = (in_dim.0 / in_size * out_size, in_dim.1 / in_size * out_size);
    let out_img = RefCell::new(image::RgbaImage::new(out_dim.0, out_dim.1));
    //let out_img = RefCell::new(image::RgbaImage::new(out_dim.0*2, out_dim.1*2));
    
    let downscale_tile = |tx, ty, offset : (u32, u32)|
    {
        let mut out = image::RgbaImage::new(out_size, out_size);
        let in_x  = tx * in_size;
        let in_y  = ty * in_size;
        for iy in 0..out_size
        {
            for ix in 0..out_size
            {
                let px = in_img.borrow().get_pixel(in_x+ix*2 + offset.0, in_y+iy*2 + offset.1).to_rgba();
                out.put_pixel(ix, iy, px);
            }
        }
        out
    };
    
    let write_tile = |tx, ty, tile : &image::RgbaImage|
    {
        for iy in 0..out_size
        {
            for ix in 0..out_size
            {
                let px = tile.get_pixel(ix, iy).to_rgba();
                out_img.borrow_mut().put_pixel(tx*out_size + ix, ty*out_size + iy, px);
            }
        }
    };
    
    let measure_tile = |tx, ty, size, img : &image::RgbaImage|
    {
        let luma = |rgba : &image::Rgba<u8>| rgba.to_luma().channels()[0] as f32;
        
        let mut avg = 0.0;
        
        for iy in 0..size
        {
            for ix in 0..size
            {
                avg += luma(&img.get_pixel(ix, iy).to_rgba());
            }
        }
        avg /= (size*size) as f32;
        
        let mut rms = 0.0;
        
        for iy in 0..size
        {
            for ix in 0..size
            {
                let px = luma(&img.get_pixel(ix, iy).to_rgba()) - avg;
                rms += px * px;
            }
        }
        rms = rms.powf(0.5);
        (avg, rms)
    };
    
    let dist2 = |a : (f32, f32), b : (f32, f32)| OrderedFloat::from((a.0-b.0).powf(2.0) + (a.1-b.1).powf(2.0));
    
    for iy in 0..in_dim.1/in_size
    {
        for ix in 0..in_dim.0/in_size
        {
            let mut tiles = Vec::new();
            tiles.push(downscale_tile(ix, iy, (0, 0)));
            tiles.push(downscale_tile(ix, iy, (0, 1)));
            tiles.push(downscale_tile(ix, iy, (1, 0)));
            tiles.push(downscale_tile(ix, iy, (1, 1)));
            
            let m = measure_tile(ix, iy, in_size, &in_img.borrow());
            tiles.sort_by_key(|tile|
            {
                let m2 = measure_tile(ix, iy, out_size, tile);
                dist2(m, m2)
            });
            write_tile(ix, iy, &tiles[0]);
        }
    }
    
    let mut out_img = out_img.into_inner();
    out_img = image::imageops::resize(&out_img, out_dim.0*2, out_dim.1*2, image::imageops::FilterType::Nearest);
    out_img.save(out_filename).unwrap();
}
