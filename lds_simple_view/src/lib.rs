#![deny(
    // missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use image::{ImageBuffer, LumaA};

use lazy_diamond_square as lds;
use lds::HeightMap;

/// When the ´gen_img´ function is called, an image will be 
/// created. The lighter the pixel, the higher the height value 
/// at that point. The image will have the ´LumaA´ color 
/// space from the 'image' crate. Locations with a height 
/// value of 'None' will have a default value. 
/// Warning! Do not use this function with instances of the 
/// ´HeightMap´ structure whose map size is very large! The 
/// output image will be 'map.size()'. 
/// Note: when specifying the `name` parameter, add 
/// the output image type, e.g. `"img.png"`.
pub fn gen_img(map: &HeightMap, name: &str) {
    let size = map.size() as u32;

    let mut cur: Option<f32>;

    let mut img: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::new(size, size);

    for y in 0..size {
        for x in 0..size {
            cur = map.get(x as i32, y as i32);

            if let Some(cur_h) = cur {
                img.put_pixel(x, y, LumaA([(255.0 * cur_h) as u8, 255]))
            }
        }
    }

    img.save(name).unwrap();
}
