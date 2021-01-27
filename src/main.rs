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
    
    let median_tile = |tx : u32, ty : u32|
    {
        let mut out = image::RgbaImage::new(out_size, out_size);
        let in_x  = tx * in_size;
        let in_y  = ty * in_size;
        let out_x = tx * out_size;
        let out_y = ty * out_size;
        for iy in 0..out_size
        {
            for ix in 0..out_size
            {
                let mut px = Vec::new();
                let mut push = |px : &mut Vec<_>, x : i64, y : i64|
                {
                    let mut x = in_x as i64 + ix as i64*2 + x;
                    let mut y = in_y as i64 + iy as i64*2 + y;
                    //if x < in_x as i64 || x >= (in_x + in_size) as i64 || y < in_y as i64 || y >= (in_y + in_size) as i64
                    //{
                    //    return;
                    //}
                    if x < in_x as i64
                    {
                        x += in_size as i64;
                    }
                    if x >= (in_x + in_size) as i64
                    {
                        x -= in_size as i64;
                    }
                    if y < in_y as i64
                    {
                        y += in_size as i64;
                    }
                    if y >= (in_y + in_size) as i64
                    {
                        y -= in_size as i64;
                    }
                    let pixel = in_img.borrow().get_pixel(x.try_into().unwrap(), y.try_into().unwrap()).to_rgba();
                    if pixel.channels()[3] < 127 { return; }
                    px.push(pixel);
                };
                for x in -1..=2
                {
                    for y in -1..=2
                    {
                        push(&mut px, x, y);
                    }
                }
                if px.len() == 0
                {
                    px.push(in_img.borrow().get_pixel(in_x+ix*2, in_y+iy*2).to_rgba());
                }
                
                let luma = |rgba : &image::Rgba<u8>| rgba.to_luma().channels()[0];
                px.sort_by_key(luma);
                let mut px_luma = px.iter().map(|x| luma(x) as f32).collect::<Vec<_>>();
                
                let mid_luma = 
                if px_luma.len() % 2 == 1
                {
                    px_luma[px_luma.len()/2]
                }
                else
                {
                    (px_luma[px_luma.len()/2] + px_luma[px_luma.len()/2-1])/2.0
                };
                
                let mut polarity = 0;
                if (px_luma[0] - mid_luma).abs() > (px_luma[px.len()-1] - mid_luma).abs()
                {
                    polarity = 1;
                }
                else if (px_luma[0] - mid_luma).abs() < (px_luma[px.len()-1] - mid_luma).abs()
                {
                    polarity = -1;
                }
                
                px = Vec::new();
                for x in 0..=1
                {
                    for y in 0..=1
                    {
                        push(&mut px, x, y);
                    }
                }
                if px.len() == 0
                {
                    px.push(in_img.borrow().get_pixel(in_x+ix*2, in_y+iy*2).to_rgba());
                }
                
                px.sort_by_key(luma);
                let mut px_luma = px.iter().map(|x| luma(x) as f32).collect::<Vec<_>>();
                
                
                let out_px;
                if polarity == 0
                {
                    out_px = 
                    if px.len() % 2 == 1
                    {
                        px[px.len()/2]
                    }
                    else
                    {
                        let a = px[px.len()/2].channels();
                        let b = px[px.len()/2-1].channels();
                        image::Rgba::<u8>::from_channels(
                            ((a[0] as u32+b[0] as u32)/2) as u8,
                            ((a[1] as u32+b[1] as u32)/2) as u8,
                            ((a[2] as u32+b[2] as u32)/2) as u8,
                            ((a[3] as u32+b[3] as u32)/2) as u8)
                    };
                }
                else
                {
                    let max = px.len() as f32 - 1.0;
                    let mut mid = 0.5;
                    if polarity == 1
                    {
                        mid -= 0.5;
                    }
                    else
                    {
                        mid += 0.5;
                    }
                    let position = mid*max;
                    out_px = px[position as usize];
                }
                
                out.put_pixel(ix, iy, out_px);
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
            tiles.push(median_tile(ix, iy));
            
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
