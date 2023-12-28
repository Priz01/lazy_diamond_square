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

use std::fmt;

use seahash::{hash, hash_seeded};
use tinyrand::{Rand, Seeded, StdRand};
use tinyrand_std::clock_seed::ClockSeed;

#[cfg(feature = "simple_viewing")]
pub use image::ImageBuffer;
#[cfg(feature = "simple_viewing")]
use image::LumaA;

const MIN_SIZE_SHIFT: u8 = 3;
/// The constant with the value of the minimum map size. If you
/// specify the `size` parameter less than this constant when
/// calling `HeightMap::new`, it will be changed to equal to
/// this constant.
///
/// # Examples
///
/// ```
/// assert_eq!(MIN_SIZE, 9)
/// ```
pub const MIN_SIZE: i32 = (1 << MIN_SIZE_SHIFT) + 1;

const MAX_SIZE_SHIFT: u8 = 29;
/// The constant with the value of the maximum map size. If
/// when calling `HeightMap::new` you specify the `size`
/// parameter greater than this constant, it will change to
/// equal to this constant.
///
/// # Examples
///
/// ```
/// assert_eq!(MAX_SIZE, 536870913)
/// ```
pub const MAX_SIZE: i32 = (1 << MAX_SIZE_SHIFT) + 1;

/// This structure is the main structure in this crate.
pub struct HeightMap {
    map: Vec<Option<f32>>,
    size: i32,
    roughness: f32,
    seed: u64,
    use_clock_seed: bool,
    gen_rand_fn: Box<dyn Fn(i32, i32, u64) -> u64>,
    calc_roughness_fn: Box<dyn Fn(i32, i32, f32) -> f32>,
    change_calced_h_fn: Box<dyn Fn(i32, i32, f32) -> f32>,
}

impl HeightMap {
    /// Returns a new instance of the `HeightMap` structure.
    pub fn new(size: i32, roughness: f32) -> Self {
        let mut map = HeightMap::default();
        map.set_size(size);
        map.set_roughness(roughness);
        map.map = vec![None; (map.size() * map.size()) as usize];
        map.init(1, InitBy::DiamondSquare);

        map
    }
    /// Returns a new seeded instance of the `HeightMap` structure.
    pub fn new_with_seed(size: i32, roughness: f32, seed: &str) -> HeightMap {
        let mut map = HeightMap::new(size, roughness);
        map.set_seed(seed);

        map
    }
    /// Getter for `size` field.
    pub fn size(&self) -> i32 {
        self.size
    }
    /// Returns the maximum value of the coordinate. If a value
    /// greater than this is passed into a function such as `get`, it
    /// will change inside the function to a valid value that is less
    /// than or equal to this value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_diamond_square as lds;
    /// use lds::{HeightMap, MIN_SIZE};
    ///
    /// let map = HeightMap::new(MIN_SIZE, 0.15);
    ///
    /// assert_eq!(map.max_coord(), map.size() - 1);
    /// ```
    pub fn max_coord(&self) -> i32 {
        self.size() - 1
    }
    /// Getter for `roughness` field.
    pub fn roughness(&self) -> f32 {
        self.roughness
    }
    /// Getter for `seed` field.
    pub fn seed(&self) -> u64 {
        self.seed
    }
    /// Getter for `use_clock_seed` field.
    pub fn use_clock_seed(&self) -> bool {
        self.use_clock_seed
    }

    fn set_size(&mut self, size: i32) {
        if (MIN_SIZE..=MAX_SIZE).contains(&size) {
            let mut last_val = 0;
            let mut val;
            for i in MIN_SIZE_SHIFT..=MAX_SIZE_SHIFT {
                val = (1 << i) + 1;
                if size > val {
                    last_val = val;
                    continue;
                } else if size == val {
                    self.size = size;
                    break;
                } else if size < val && size > last_val {
                    if size < val >> 1 {
                        self.size = last_val;
                        break;
                    } else {
                        self.size = val;
                        break;
                    }
                }
            }
        } else if size < MIN_SIZE {
            self.size = MIN_SIZE;
        } else {
            self.size = MAX_SIZE;
        }
    }

    fn set_seed(&mut self, seed: &str) {
        self.seed = hash(seed.as_bytes());
    }

    fn set_roughness(&mut self, roughness: f32) {
        self.roughness = if roughness > 1.0 {
            1.0
        } else {
            roughness.abs()
        };
    }
    /// Returns the height value at specified coordinates. If
    /// this value exceeds the range `0..=self.max_coord()`, it is
    /// changed to valid coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_diamond_square as lds;
    /// use lds::{HeightMap, MIN_SIZE};
    ///
    /// let map = HeightMap::new(MIN_SIZE, 0.15);
    ///
    /// assert_ne!(map.get(0, 0), None);
    /// assert_eq!(map.get(1, 0), None);
    /// assert_ne!(map.get(map.max_coord() + 1, 0), None);
    /// ```
    pub fn get(&self, x: i32, y: i32) -> Option<f32> {
        let (x, y) = self.to_valid_coords(x, y);

        self.map[(y * self.size() + x) as usize]
    }
    /// Sets the passed value at specified coordinates and
    /// returns the value that was there before. If this coordinates
    /// leaves the range `0..=self.max_coord()`, this coordinates are
    /// changed to valid coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_diamond_square as lds;
    /// use lds::{HeightMap, MIN_SIZE};
    ///
    /// let mut map = HeightMap::new(MIN_SIZE, 0.15);
    ///
    /// assert_ne!(map.set(0, 0, Some(0.5)), None);
    /// assert_eq!(map.set(1, 0, Some(0.5)), None);
    ///
    /// assert_eq!(map.get(0, 0), Some(0.5));
    /// assert_eq!(map.get(1, 0), Some(0.5));
    /// ```
    pub fn set(&mut self, x: i32, y: i32, h: Option<f32>) -> Option<f32> {
        let (x, y) = self.to_valid_coords(x, y);

        let old = self.get(x, y);

        let index = (y * self.size() + x) as usize;
        self.map[index] = h;

        old
    }
    /// Checks the value against the specified coordinates, and if
    /// the value is `None`, then generates the new height value,
    /// sets it and returns the same value and `true`. Otherwise,
    /// returns the resulting height value and `false`. Also
    /// returns `(None, false)` if the value cannot be generated
    /// because at least one corner of the map is set to `None`. If
    /// this coordinate value is outside the range `0..=self.max_coord()`,
    /// this coordinates are changed to valid coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_diamond_square as lds;
    /// use lds::{HeightMap, MIN_SIZE};
    ///
    /// let mut map = HeightMap::new(MIN_SIZE, 0.15);
    ///
    /// map.set(1, 0, Some(0.5));
    ///
    /// assert_eq!(map.gen(0, 0), map.get(0, 0));
    /// assert_eq!(map.gen(1, 0), map.get(1, 0));
    /// assert_eq!(map.gen(2, 0), map.get(2, 0));
    ///
    /// assert_ne!(map.get(0, 0), None);
    /// assert_eq!(map.get(1, 0), Some(0.5));
    /// assert_ne!(map.get(2, 0), None);
    /// ```
    pub fn gen(&mut self, x: i32, y: i32) -> Option<f32> {
        let mut h = None;

        let mut coords: Vec<[i32; 2]> = vec![[x, y]];

        let mut last_coords = *coords.last().unwrap();

        let max_coord = self.max_coord();

        if (last_coords[0] == max_coord || last_coords[0] == 0)
            && (last_coords[1] == max_coord || last_coords[1] == 0)
        {
            h = self.get(x, y)
        } else if let Some(height) = self.get(x, y) {
            h = Some(height);
        } else {
            let mut indexes: Vec<u8> = vec![0];
            let mut heights: Vec<[f32; 4]> = vec![[0.0; 4]];

            let mut last_index = *indexes.last().unwrap();
            let mut last_heights = *heights.last().unwrap();
            let mut len = coords.len();
            let mut step = self.calc_step(x, y);

            let mut diamond_step: bool = false;

            if ((x & step) != 0) && ((y & step) != 0) {
                diamond_step = true;
            }

            while len != 0 {
                let (x, y) = self.calc_coords(
                    last_coords[0],
                    last_coords[1],
                    step,
                    last_index,
                    diamond_step,
                );
                let (x, y) = self.to_valid_coords(x, y);

                if let Some(height) = self.get(x, y) {
                    last_heights[last_index as usize] = height;
                    heights.pop();
                    heights.push(last_heights);

                    if last_index != 3 {
                        indexes.pop();
                        indexes.push(last_index + 1);
                    } else {
                        indexes.pop();
                        coords.pop();
                        heights.pop();

                        if len == 1 {
                            let height = Some(self.calc_h(x, y, last_heights));

                            self.set(last_coords[0], last_coords[1], height);

                            h = height;
                        } else {
                            self.set(
                                last_coords[0],
                                last_coords[1],
                                Some(self.calc_h(x, y, last_heights)),
                            );
                        }

                        len = coords.len();

                        if len != 0 {
                            last_coords = *coords.last().unwrap();
                        }

                        diamond_step = !diamond_step;
                    }
                } else {
                    indexes.push(0);
                    coords.push([x, y]);
                    heights.push([0.0; 4]);

                    last_coords = *coords.last().unwrap();
                    len = coords.len();

                    step = self.calc_step(x, y);

                    diamond_step = ((x & step) != 0) && ((y & step) != 0);
                }

                if len != 0 {
                    last_index = *indexes.last().unwrap();
                    last_heights = *heights.last().unwrap();
                }
            }
        }

        h
    }
    /// Returns a vector of the results of calls to the `get`
    /// method for each point on the specified area.
    pub fn get_area(
        &self,
        top_left: (i32, i32),
        bottom_right: (i32, i32),
    ) -> Vec<(i32, i32, Option<f32>)> {
        let mut area = Vec::with_capacity(self.size().pow(2) as usize);
        let (top_left, bottom_right) = (
            self.to_valid_coords(top_left.0, top_left.1),
            self.to_valid_coords(bottom_right.0, bottom_right.1),
        );

        for y in top_left.1..bottom_right.1 {
            for x in top_left.0..bottom_right.0 {
                area.push((x, y, self.get(x, y)));
            }
        }

        area
    }
    /// Returns a vector of the results of calls to the `set`
    /// method for each point on the specified area.
    pub fn set_area(
        &mut self,
        h: f32,
        top_left: (i32, i32),
        bottom_right: (i32, i32),
    ) -> Vec<(i32, i32, Option<f32>)> {
        let mut old_area = Vec::with_capacity(self.size().pow(2) as usize);
        let (top_left, bottom_right) = (
            self.to_valid_coords(top_left.0, top_left.1),
            self.to_valid_coords(bottom_right.0, bottom_right.1),
        );

        for y in top_left.1..bottom_right.1 {
            for x in top_left.0..bottom_right.0 {
                old_area.push((x, y, self.set(x, y, Some(h))));
            }
        }

        old_area
    }
    /// Returns a vector of the results of calls to the `gen`
    /// method for each point on the specified area.
    pub fn gen_area(
        &mut self,
        top_left: (i32, i32),
        bottom_right: (i32, i32),
    ) -> Vec<(i32, i32, Option<f32>)> {
        let mut area = Vec::with_capacity(self.size().pow(2) as usize);
        let (top_left, bottom_right) = (
            self.to_valid_coords(top_left.0, top_left.1),
            self.to_valid_coords(bottom_right.0, bottom_right.1),
        );

        for y in top_left.1..bottom_right.1 {
            for x in top_left.0..bottom_right.0 {
                area.push((x, y, self.gen(x, y)));
            }
        }

        area
    }
    /// The lighter the pixel, the higher the height value at
    /// that point. Locations with a height value of 'None' will
    /// have a default value. To save use 'save' or
    /// 'save_with_format' methods.
    ///
    /// Creates an image with passed file name and extension, if called.
    #[cfg(feature = "simple_viewing")]
    pub fn get_img(
        &self,
        top_left: (i32, i32),
        bottom_right: (i32, i32),
        name: Option<&str>,
    ) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
        let (top_left, bottom_right) = (
            self.to_valid_coords(top_left.0, top_left.1),
            self.to_valid_coords(bottom_right.0, bottom_right.1),
        );

        let mut img: ImageBuffer<LumaA<u8>, Vec<u8>>;
        {
            let size = (bottom_right.0 - top_left.0) as u32;
            img = ImageBuffer::new(size, size);
        }

        let mut cur: Option<f32>;

        for y in top_left.1 as u32..bottom_right.1 as u32 {
            for x in top_left.0 as u32..bottom_right.0 as u32 {
                cur = self.get(x as i32, y as i32);

                if let Some(cur_h) = cur {
                    img.put_pixel(
                        x - top_left.0 as u32,
                        y - top_left.1 as u32,
                        LumaA([(255.0 * cur_h) as u8, 255]),
                    )
                }
            }
        }

        if let Some(img_name) = name {
            img.save(img_name).unwrap();
        }

        img
    }

    fn init(&mut self, lvl: u8, init_by: InitBy) {
        let mut lvl = lvl;
        {
            let max_lvl = (self.max_coord().trailing_zeros()) as u8;

            if lvl > max_lvl {
                lvl = max_lvl;
            }
        }

        let max_coord = self.max_coord();
        let size = self.size();
        let seed = self.seed();

        let a = seed & 0xFFFF;
        let b = seed & 0xFFFF0000;
        let c = seed & 0xFFFF00000000;
        let d = seed & 0xFFFF000000000000;

        match init_by {
            InitBy::DiamondSquare => {
                {
                    let corners = [
                        (0, 0),
                        (max_coord, 0),
                        (0, max_coord),
                        (max_coord, max_coord),
                    ];
                    let (mut x, mut y);

                    for corner in corners {
                        (x, y) = (corner.0, corner.1);
                        self.set(
                            x,
                            y,
                            Some(Self::to_range(
                                0.0,
                                1.0,
                                hash_seeded(
                                    (x.to_string() + "_" + &y.to_string()).as_bytes(),
                                    a,
                                    b,
                                    c,
                                    d,
                                ) as u16,
                            )),
                        );
                    }
                }

                let mut step = max_coord;
                let mut shift = step >> 1;

                let mut x = 0;
                let mut y = 0;

                for _ in 0..lvl {
                    y += shift;
                    x += shift;

                    while y < size {
                        while x < size {
                            let h = self.calc_h(
                                x,
                                y,
                                [
                                    self.get(x + shift, y - shift).unwrap(),
                                    self.get(x + shift, y + shift).unwrap(),
                                    self.get(x - shift, y + shift).unwrap(),
                                    self.get(x - shift, y - shift).unwrap(),
                                ],
                            );

                            self.set(x, y, Some(h));

                            x += step;
                        }

                        x = step >> 1;
                        y += step;
                    }

                    x = 0;
                    y = 0;

                    step >>= 1;

                    while y < size {
                        while x < size {
                            if self.get(x, y).is_none() {
                                let h = self.calc_h(
                                    x,
                                    y,
                                    [
                                        self.get_for_square(x, y - shift).unwrap(),
                                        self.get_for_square(x + shift, y).unwrap(),
                                        self.get_for_square(x, y + shift).unwrap(),
                                        self.get_for_square(x - shift, y).unwrap(),
                                    ],
                                );

                                self.set(x, y, Some(h));
                            }

                            x += step;
                        }
                        x = 0;
                        y += step;
                    }

                    y = 0;

                    shift >>= 1;
                }
            }
            InitBy::Seed => {
                let step = if lvl > 1 { max_coord >> (lvl - 1) } else { 1 };

                let mut x = 0;
                let mut y = 0;
                let mut num_of_point = 0;

                while y < size {
                    while x < size {
                        self.set(
                            x,
                            y,
                            Some(Self::to_range(
                                0.0,
                                1.0,
                                hash_seeded(num_of_point.to_string().as_bytes(), a, b, c, d) as u16,
                            )),
                        );

                        x += step;
                        num_of_point += 1;
                    }
                    x = 0;
                    y += step;
                }
            }
            InitBy::None => (),
        }
    }

    fn to_valid_coords(&self, x: i32, y: i32) -> (i32, i32) {
        if x >= self.size() || y >= self.size() || x < 0 || y < 0 {
            let mut x = x;
            let mut y = y;

            let size = self.size();

            while x < 0 {
                x += size;
            }

            x %= size;

            while y < 0 {
                y += size;
            }

            y %= size;

            (x, y)
        } else {
            (x, y)
        }
    }

    fn to_range(min: f32, max: f32, h: u16) -> f32 {
        let old_range = (u16::MAX as i32 - u16::MIN as i32) as f32;
        let new_range = max - min;

        let mut result = (h as i32 - u16::MIN as i32) as f32;

        result /= old_range;
        result *= new_range;
        result += min;

        result
    }

    fn calc_h(&self, x: i32, y: i32, heights: [f32; 4]) -> f32 {
        let h = {
            let sum = heights[0] + heights[1] + heights[2] + heights[3];
            sum / 4.0
        };

        let rand = {
            if self.use_clock_seed() {
                StdRand::seed(ClockSeed.next_u64()).next_u16()
            } else {
                StdRand::seed((self.gen_rand_fn)(x, y, self.seed())).next_u16()
            }
        };

        let rand = Self::to_range(0.0, 1.0, rand);
        let r = (self.calc_roughness_fn)(x, y, self.roughness());

        (self.change_calced_h_fn)(x, y, (r * rand) + (1.0 - r) * h)
    }

    fn get_for_square(&self, x: i32, y: i32) -> Option<f32> {
        let (x, y) = self.to_valid_coords_for_square(x, y);

        self.map[(y * self.size() + x) as usize]
    }

    fn calc_step(&self, x: i32, y: i32) -> i32 {
        let mut step = 1;

        while ((x & step) == 0) && ((y & step) == 0) {
            step <<= 1;
        }

        step
    }

    fn calc_coords(&self, x: i32, y: i32, step: i32, index: u8, diamond_step: bool) -> (i32, i32) {
        let (mut x, mut y) = (x, y);

        if let 0 = index {
            if diamond_step {
                (x, y) = (x + step, y - step)
            } else {
                (x, y) = (x, y - step)
            }
        } else if let 1 = index {
            if diamond_step {
                (x, y) = (x + step, y + step)
            } else {
                (x, y) = (x + step, y)
            }
        } else if let 2 = index {
            if diamond_step {
                (x, y) = (x - step, y + step)
            } else {
                (x, y) = (x, y + step)
            }
        } else if diamond_step {
            (x, y) = (x - step, y - step)
        } else {
            (x, y) = (x - step, y)
        }

        if !diamond_step {
            if x < 0 {
                x -= 1;
            } else if x >= self.size() {
                x += 1;
            }

            if y < 0 {
                y -= 1;
            } else if y >= self.size() {
                y += 1;
            }
        }

        (x, y)
    }

    fn to_valid_coords_for_square(&self, x: i32, y: i32) -> (i32, i32) {
        let (mut x, mut y) = (x, y);

        let max_coord = self.max_coord();

        if x < 0 {
            x -= 1;
        } else if x > max_coord {
            x += 1;
        }

        if y < 0 {
            y -= 1;
        } else if y > max_coord {
            y += 1
        }

        self.to_valid_coords(x, y)
    }
}

impl Default for HeightMap {
    fn default() -> Self {
        Self {
            map: vec![],
            size: MIN_SIZE,
            roughness: 0.0,
            seed: StdRand::seed(ClockSeed.next_u64()).next_u64(),
            use_clock_seed: false,
            gen_rand_fn: Box::new(|x: i32, y: i32, seed: u64| {
                seed ^ {
                    let mut x = x as u64;
                    let mut y = y as u64;

                    let mut xm7: u64 = x % 7;
                    let mut xm13: u64 = x % 13;
                    let mut xm1301081: u64 = x % 1301081;
                    let mut ym8461: u64 = y % 8461;
                    let mut ym105467: u64 = y % 105467;
                    let mut ym105943: u64 = y % 105943;

                    for _ in 0..80 {
                        y = x + seed;
                        x += xm7 + xm13 + xm1301081 + ym8461 + ym105467 + ym105943;
                        xm7 = x % 7;
                        xm13 = x % 13;
                        xm1301081 = x % 1301081;
                        ym8461 = y % 8461;
                        ym105467 = y % 105467;
                        ym105943 = y % 105943;
                    }

                    ((xm7 + xm13 + xm1301081 + ym8461 + ym105467 + ym105943) as f64 / 1520972.0)
                        .to_bits()
                }
            }),
            calc_roughness_fn: Box::new(|_x: i32, _y: i32, r: f32| r),
            change_calced_h_fn: Box::new(|_x: i32, _y: i32, h: f32| h),
        }
    }
}

impl fmt::Debug for HeightMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeightMap")
            .field("map", &self.map)
            .field("size", &self.size)
            .field("seed", &self.seed)
            .field("roughness", &self.roughness)
            .field("use_clock_seed", &self.use_clock_seed)
            .finish()
    }
}
/// For more flexible customization of `HeightMap` parameters.
/// The names of the methods correspond to the names of the
/// fields to be set.
pub struct Builder {
    size: i32,
    seed: u64,
    roughness: f32,
    use_clock_seed: bool,
    gen_rand_fn: Box<dyn Fn(i32, i32, u64) -> u64>,
    calc_roughness_fn: Box<dyn Fn(i32, i32, f32) -> f32>,
    change_calced_h_fn: Box<dyn Fn(i32, i32, f32) -> f32>,

    init_lvl: u8,
    init_by: InitBy,
}

impl Builder {
    pub fn new(size: i32, roughness: f32) -> Builder {
        let mut self_size = size;
        let mut self_roughness = roughness.abs();
        if self_roughness > 1.0 {
            self_roughness = 1.0;
        }

        if (MIN_SIZE..=MAX_SIZE).contains(&size) {
            let mut last_val = 0;
            let mut val;
            for i in MIN_SIZE_SHIFT..=MAX_SIZE_SHIFT {
                val = (1 << i) + 1;
                if size > val {
                    last_val = val;
                    continue;
                } else if size == val {
                    self_size = size;
                    break;
                } else if size < val && size > last_val {
                    if size < val >> 1 {
                        self_size = last_val;
                        break;
                    } else {
                        self_size = val;
                        break;
                    }
                }
            }
        } else if size < MIN_SIZE {
            self_size = MIN_SIZE;
        } else {
            self_size = MAX_SIZE;
        }

        Builder {
            size: self_size,
            roughness: self_roughness,
            ..Default::default()
        }
    }
    pub fn seed(self, seed: &str) -> Self {
        Self {
            seed: hash(seed.as_bytes()),
            ..self
        }
    }
    pub fn use_clock_seed(self, by_clock: bool) -> Self {
        Self {
            use_clock_seed: by_clock,
            ..self
        }
    }
    /// The closure set by this method are further used to
    /// generate random numbers during height generation
    /// at a particular point. No effect if you set
    /// `use_clock_seed` to `true`.
    pub fn gen_rand_fn(self, f: Box<dyn Fn(i32, i32, u64) -> u64>) -> Self {
        Self {
            gen_rand_fn: f,
            ..self
        }
    }
    pub fn calc_roughness_fn(self, f: Box<dyn Fn(i32, i32, f32) -> f32>) -> Self {
        Self {
            calc_roughness_fn: f,
            ..self
        }
    }
    pub fn change_calced_h_fn(self, f: Box<dyn Fn(i32, i32, f32) -> f32>) -> Self {
        Self {
            change_calced_h_fn: f,
            ..self
        }
    }
    pub fn init_lvl(self, lvl: u8) -> Self {
        Self {
            init_lvl: lvl,
            ..self
        }
    }
    pub fn init_by(self, by: InitBy) -> Self {
        Self {
            init_by: by,
            ..self
        }
    }
    pub fn build<'a>(self) -> HeightMap {
        let mut map = HeightMap {
            map: vec![None; (self.size * self.size) as usize],
            size: self.size,
            roughness: self.roughness,
            seed: self.seed,
            use_clock_seed: self.use_clock_seed,
            gen_rand_fn: self.gen_rand_fn,
            calc_roughness_fn: self.calc_roughness_fn,
            change_calced_h_fn: self.change_calced_h_fn,
        };

        map.init(self.init_lvl, self.init_by);

        map
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            size: MIN_SIZE,
            seed: StdRand::seed(ClockSeed.next_u64()).next_u64(),
            roughness: 0.0,
            use_clock_seed: false,
            gen_rand_fn: Box::new(|x: i32, y: i32, seed: u64| {
                seed ^ {
                    let mut x = x as u64;
                    let mut y = y as u64;

                    let mut xm7: u64 = x % 7;
                    let mut xm13: u64 = x % 13;
                    let mut xm1301081: u64 = x % 1301081;
                    let mut ym8461: u64 = y % 8461;
                    let mut ym105467: u64 = y % 105467;
                    let mut ym105943: u64 = y % 105943;

                    for _ in 0..80 {
                        y = x + seed;
                        x += xm7 + xm13 + xm1301081 + ym8461 + ym105467 + ym105943;
                        xm7 = x % 7;
                        xm13 = x % 13;
                        xm1301081 = x % 1301081;
                        ym8461 = y % 8461;
                        ym105467 = y % 105467;
                        ym105943 = y % 105943;
                    }

                    ((xm7 + xm13 + xm1301081 + ym8461 + ym105467 + ym105943) as f64 / 1520972.0)
                        .to_bits()
                }
            }),
            calc_roughness_fn: Box::new(|_x: i32, _y: i32, r: f32| r),
            change_calced_h_fn: Box::new(|_x: i32, _y: i32, h: f32| h),

            init_lvl: 1,
            init_by: InitBy::DiamondSquare,
        }
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("size", &self.size)
            .field("seed", &self.seed)
            .field("roughness", &self.roughness)
            .field("use_clock_seed", &self.use_clock_seed)
            .field("init_lvl", &self.init_lvl)
            .field("init_by", &self.init_by)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// This is an enumeration with possible options for
/// initializing a height map of an instance of the
/// `HeightMap` structure.
pub enum InitBy {
    DiamondSquare,
    Seed,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_coord() {
        let map = HeightMap::new(MIN_SIZE, 0.15);

        assert_eq!(map.max_coord(), map.size() - 1);
    }
    #[test]
    fn get() {
        let map = HeightMap::new(MIN_SIZE, 0.15);

        assert_ne!(map.get(0, 0), None);
        assert_eq!(map.get(1, 0), None);
        assert_ne!(map.get(map.max_coord() + 1, 0), None);
    }

    #[test]
    fn set() {
        let mut map = HeightMap::new(MIN_SIZE, 0.15);

        assert_ne!(map.set(0, 0, Some(0.5)), None);
        assert_eq!(map.set(1, 0, Some(0.5)), None);

        assert_eq!(map.get(0, 0), Some(0.5));
        assert_eq!(map.get(1, 0), Some(0.5));
    }

    #[test]
    fn gen() {
        let mut map = HeightMap::new(MIN_SIZE, 0.15);

        map.set(1, 0, Some(0.5));

        assert_eq!(map.gen(0, 0), map.get(0, 0));
        assert_eq!(map.gen(1, 0), map.get(1, 0));
        assert_eq!(map.gen(2, 0), map.get(2, 0));

        assert_ne!(map.get(0, 0), None);
        assert_eq!(map.get(1, 0), Some(0.5));
        assert_ne!(map.get(2, 0), None);
    }
}
