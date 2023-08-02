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

use Parameters::*;

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

#[derive()]
/// This structure is the main structure in this crate. 
pub struct HeightMap {
    map: Vec<Option<f32>>,
    size: i32,
    seed: u64,
    roughness: f32,
    calc_roughness_fn: Box<dyn Fn(i32, i32, f32) -> f32>,
    change_calced_h_fn: Box<dyn Fn(i32, i32, f32) -> f32>,
}

impl HeightMap {
    /// Returns a new instance of the `HeightMap` structure. 
    /// Optionally you can pass some parameters such as seed, 
    /// initialization level or other from the Parameters 
    /// enumeration. 
    pub fn new(size: i32, roughness: f32, param: Vec<Parameters>) -> Self {
        let mut map = Self {
            map: vec![None; (size * size) as usize],
            ..Default::default()
        };

        map.set_size(size);
        map.set_roughness(roughness);

        let mut lvl: u8 = 1;
        let mut init_by = InitBy::DiamondSquare;

        for p in param {
            match p {
                Seed(seed) => map.set_seed(seed),
                InitLevel(level) => lvl = level,
                InitBy(by) => init_by = by,
                CalcRoughnessFn(func) => map.set_calc_roughness_fn(func),
                ChangeCalcedHeightFn(func) => map.set_change_calced_h_fn(func),
            }
        }

        map.init(lvl, init_by);

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
    /// let map = HeightMap::new(MIN_SIZE, 0.15, vec![]);
    /// 
    /// assert_eq!(map.max_coord(), map.size() - 1);
    /// ```
    pub fn max_coord(&self) -> i32 {
        self.size() - 1
    }
    /// Getter for `seed` field. 
    pub fn seed(&self) -> u64 {
        self.seed
    }
    /// Getter for `roughness` field. 
    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    fn set_size(&mut self, size: i32) {
        if size >= MIN_SIZE && size <= MAX_SIZE {
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

    fn set_calc_roughness_fn(&mut self, func: Box<dyn Fn(i32, i32, f32) -> f32>) {
        self.calc_roughness_fn = func;
    }

    fn set_change_calced_h_fn(&mut self, func: Box<dyn Fn(i32, i32, f32) -> f32>) {
        self.change_calced_h_fn = func;
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
    /// let map = HeightMap::new(MIN_SIZE, 0.15, vec![]);
    /// 
    /// assert_ne!(map.get(0, 0), None);
    /// assert_eq!(map.get(1, 0), None);
    /// assert_ne!(map.get(map.max_coord() + 1, 0), None); // or assert_ne!(map.get(map.size(), 0), None)
    /// ```
    pub fn get(&self, x: i32, y: i32) -> Option<f32> {
        let (x, y) = self.to_valid_coords(x, y);

        let index: usize = (y * self.size() + x) as usize;
        self.map[index]
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
    /// let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);
    /// 
    /// assert_ne!(map.set(0, 0, 0.5), None);
    /// assert_eq!(map.set(1, 0, 0.5), None);
    /// 
    /// assert_eq!(map.get(0, 0), Some(0.5));
    /// assert_eq!(map.get(1, 0), Some(0.5));
    /// ```
    pub fn set(&mut self, x: i32, y: i32, h: f32) -> Option<f32> {
        let (x, y) = self.to_valid_coords(x, y);

        let index: usize = (y * self.size() + x) as usize;

        let old = self.get(x, y);

        self.map[index] = Some(h);

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
    /// let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);
    /// 
    /// map.set(1, 0, 0.5);
    /// 
    /// assert_eq!(map.gen(0, 0), (map.get(0, 0), false));
    /// assert_eq!(map.gen(1, 0), (map.get(1, 0), false));
    /// assert_eq!(map.gen(2, 0), (map.get(2, 0), true));
    ///
    /// assert_ne!(map.get(0, 0), None);
    /// assert_eq!(map.get(1, 0), Some(0.5));
    /// assert_ne!(map.get(2, 0), None);
    /// ```
    pub fn gen(&mut self, x: i32, y: i32) -> (Option<f32>, bool) {
        match self.get(x, y) {
            Some(h) => (Some(h), false),
            None => match self.gen_h(x, y) {
                Some(h) => (self.set(x, y, h), true),
                None => (None, false),
            },
        }
    }
    /// Generates a new height value, sets it and returns the 
    /// old and new values. Returns old value and `None` if the value 
    /// cannot be generated because at least one corner of the 
    /// map is set to `None`. If this coordinate value is outside the 
    /// range `0..=max_coord()`, this coordinates are changed to 
    /// valid coordinates.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use lazy_diamond_square as lds;
    /// use lds::{HeightMap, MIN_SIZE};
    /// 
    /// let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);
    /// 
    /// map.set(1, 0, 0.5);
    /// 
    /// assert_eq!(map.regen(0, 0), (map.get(0, 0), map.get(0, 0)));
    /// assert_ne!(map.regen(1, 0), (map.get(1, 0), None));
    /// assert_ne!(map.regen(2, 0), (map.get(2, 0), None));
    /// 
    /// assert_ne!(map.get(0, 0), None);
    /// assert_ne!(map.get(1, 0), None);
    /// assert_ne!(map.get(2, 0), None);
    /// ```
    pub fn regen(&mut self, x: i32, y: i32) -> (Option<f32>, Option<f32>) {
        match self.gen_h(x, y) {
            Some(h) => (self.set(x, y, h), Some(h)),
            None => (None, None),
        }
    }
    /// Returns a vector of the results of calls to the `get` 
    /// method for each point on the map. 
    pub fn get_all(&self) -> Vec<Option<f32>> {
        let mut map = Vec::with_capacity(self.size().pow(2) as usize);

        for y in 0..self.size() {
            for x in 0..self.size() {
                map.push(self.get(x, y));
            }
        }

        map
    }
    /// Returns a vector of the results of calls to the `set` 
    /// method for each point on the map. 
    pub fn set_all(&mut self, h: f32) -> Vec<Option<f32>> {
        let mut old_map = Vec::with_capacity(self.size().pow(2) as usize);

        for y in 0..self.size() {
            for x in 0..self.size() {
                old_map.push(self.set(x, y, h));
            }
        }

        old_map
    }
    /// Returns a vector of the results of calls to the `gen` 
    /// method for each point on the map. 
    pub fn gen_all(&mut self) -> Vec<(Option<f32>, bool)> {
        let mut map = Vec::with_capacity(self.size().pow(2) as usize);

        for y in 0..self.size() {
            for x in 0..self.size() {
                map.push(self.gen(x, y));
            }
        }

        map
    }
    /// When ´gen_img´ is called, an image will be created. The 
    /// lighter the pixel, the higher the height value at that point. 
    /// Locations with a height value of 'None' will have a default 
    /// value.
    /// Warning! Do not use this function with instances of 
    /// ´HeightMap´ structure whose map size is very large! The 
    /// output image size will be equal to 'map.size()'. 
    /// Note: when specifying the `name` parameter, add the 
    /// output image type, e.g. `"img.png"`. 
    #[cfg(feature = "simple_viewing")]
    pub fn gen_img(&self, name: &str) {
        use image::{ImageBuffer, LumaA};

        let size = self.size() as u32;

        let mut cur: Option<f32>;

        let mut img: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::new(size, size);

        for y in 0..size {
            for x in 0..size {
                cur = self.get(x as i32, y as i32);

                if let Some(cur_h) = cur {
                    img.put_pixel(x, y, LumaA([(255.0 * cur_h) as u8, 255]))
                }
            }
        }

        img.save(name).unwrap();
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
                            Self::to_range(
                                0.0,
                                1.0,
                                hash_seeded(
                                    (x.to_string() + "_" + &y.to_string()).as_bytes(),
                                    a,
                                    b,
                                    c,
                                    d,
                                ) as u16,
                            ),
                        );
                    }
                }

                let mut step = self.max_coord();
                let mut shift = step >> 1;

                let mut x = 0;
                let mut y = 0;

                for _ in 0..lvl {
                    y += shift;
                    x += shift;

                    while y < self.size() {
                        while x < self.size() {
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

                            self.set(x, y, h);

                            x += step;
                        }

                        x = step >> 1;
                        y += step;
                    }

                    x = 0;
                    y = 0;

                    step >>= 1;

                    while y < self.size() {
                        while x < self.size() {
                            if let None = self.get(x, y) {
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

                                self.set(x, y, h);
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
                let step = if lvl > 1 {
                    self.max_coord() >> (lvl - 1)
                } else {
                    1
                };

                let mut x = 0;
                let mut y = 0;
                let mut num_of_point = 0;

                let size = self.size();

                while y < size {
                    while x < size {
                        self.set(
                            x,
                            y,
                            Self::to_range(
                                0.0,
                                1.0,
                                hash_seeded(num_of_point.to_string().as_bytes(), a, b, c, d) as u16,
                            ),
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
            let mut seed = self.seed();

            seed ^= {
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
            };

            // StdRand::seed(ClockSeed::default().next_u64()).next_u16() // !
            StdRand::seed(seed).next_u16()
        };

        let rand = Self::to_range(0.0, 1.0, rand);
        let r = (self.calc_roughness_fn)(x, y, self.roughness());

        (self.change_calced_h_fn)(x, y, (r * rand) + (1.0 - r) * h)
    }

    fn get_for_square(&self, x: i32, y: i32) -> Option<f32> {
        let (x, y) = self.to_valid_coords_for_square(x, y);

        let index: usize = (y * self.size() + x) as usize;
        self.map[index]
    }

    fn gen_h(&mut self, x: i32, y: i32) -> Option<f32> {
        let mut h = None;

        let mut coords: Vec<[i32; 2]> = vec![[x, y]];

        let mut last_coords = *coords.last().unwrap();

        let max_coord = self.max_coord();

        if !(last_coords[0] == 0 && last_coords[1] == 0)
            && !(last_coords[0] == max_coord && last_coords[1] == 0)
            && !(last_coords[0] == 0 && last_coords[1] == max_coord)
            && !(last_coords[0] == max_coord && last_coords[1] == max_coord)
        {
            let mut indexes: Vec<u8> = vec![0];
            let mut heights: Vec<[f32; 4]> = vec![[0.0; 4]];

            let mut last_index = *indexes.last().unwrap();
            let mut last_heights = *heights.last().unwrap();
            let mut len = coords.len();
            let mut step = self.calc_step(x, y);

            let mut is_diamond: bool = false;

            if ((x & step) != 0) && ((y & step) != 0) {
                is_diamond = true;
            }

            while len != 0 {
                let (x, y) =
                    self.calc_coords(last_coords[0], last_coords[1], step, last_index, is_diamond);
                let (x, y) = self.to_valid_coords(x, y);

                match self.get(x, y) {
                    Some(height) => {
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
                                let height = self.calc_h(x, y, last_heights);
                                
                                self.set(last_coords[0], last_coords[1], height);

                                h = Some(height);
                            } else {
                                self.set(
                                    last_coords[0],
                                    last_coords[1],
                                    self.calc_h(x, y, last_heights),
                                );
                            }

                            len = coords.len();

                            if len != 0 {
                                last_coords = *coords.last().unwrap();
                            }

                            is_diamond = !is_diamond;
                        }
                    }
                    None => {
                        indexes.push(0);
                        coords.push([x, y]);
                        heights.push([0.0; 4]);

                        last_coords = *coords.last().unwrap();
                        len = coords.len();

                        step = self.calc_step(x, y);

                        if ((x & step) != 0) && ((y & step) != 0) {
                            is_diamond = true;
                        } else {
                            is_diamond = false;
                        }
                    }
                }

                if len != 0 {
                    last_index = *indexes.last().unwrap();
                    last_heights = *heights.last().unwrap();
                }
            }
        } else {
            h = self.get(x, y);
        }

        h
    }

    fn calc_step(&self, x: i32, y: i32) -> i32 {
        let mut step = 1;

        while ((x & step) == 0) && ((y & step) == 0) {
            step <<= 1;
        }

        step
    }

    fn calc_coords(&self, x: i32, y: i32, step: i32, index: u8, is_diamond: bool) -> (i32, i32) {
        let (mut x, mut y) = (x, y);

        if let 0 = index {
            if is_diamond {
                (x, y) = (x + step, y - step)
            } else {
                (x, y) = (x, y - step)
            }
        } else if let 1 = index {
            if is_diamond {
                (x, y) = (x + step, y + step)
            } else {
                (x, y) = (x + step, y)
            }
        } else if let 2 = index {
            if is_diamond {
                (x, y) = (x - step, y + step)
            } else {
                (x, y) = (x, y + step)
            }
        } else {
            if is_diamond {
                (x, y) = (x - step, y - step)
            } else {
                (x, y) = (x - step, y)
            }
        }

        if !is_diamond {
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
            seed: StdRand::seed(ClockSeed::default().next_u64()).next_u64(),
            roughness: 0.0,
            calc_roughness_fn: Box::new(|_x: i32, _y: i32, r: f32| r),
            change_calced_h_fn: Box::new(|_x: i32, _y: i32, h: f32| h),
        }
    }
}

impl fmt::Debug for HeightMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
        .field("map", &self.map)
        .field("size", &self.size)
        .field("seed", &self.seed)
        .field("roughness", &self.roughness)
        .finish()
    }
}

#[derive()]
/// This is an enumeration with possible parameters for 
/// creating an instance of the `HeightMap` structure. 
pub enum Parameters<'a> {
    Seed(&'a str),
    InitLevel(u8),
    InitBy(InitBy),
    CalcRoughnessFn(Box<dyn Fn(i32, i32, f32) -> f32>),
    ChangeCalcedHeightFn(Box<dyn Fn(i32, i32, f32) -> f32>),
}

impl<'a> fmt::Debug for Parameters<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Seed(seed) => write!(f, "{}", seed),
            InitLevel(lvl) => write!(f, "{}", lvl),
            InitBy(by) => write!(f, "{:?}", by),
            _ => write!(f, "Box<dyn Fn(i32, i32, f32) -> f32>")
        }
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
        let map = HeightMap::new(MIN_SIZE, 0.15, vec![]);

        assert_eq!(map.max_coord(), map.size() - 1);
    }
    #[test]
    fn get() {
        let map = HeightMap::new(MIN_SIZE, 0.15, vec![]);

        assert_ne!(map.get(0, 0), None);
        assert_eq!(map.get(1, 0), None);
    }

    #[test]
    fn set() {
        let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);

        assert_ne!(map.set(0, 0, 0.5), None);
        assert_eq!(map.set(1, 0, 0.5), None);

        assert_eq!(map.get(0, 0), Some(0.5));
        assert_eq!(map.get(1, 0), Some(0.5));
    }

    #[test]
    fn gen() {
        let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);

        map.set(1, 0, 0.5);
 
        assert_eq!(map.gen(0, 0), (map.get(0, 0), false));
        assert_eq!(map.gen(1, 0), (map.get(1, 0), false));
        assert_eq!(map.gen(2, 0), (map.get(2, 0), true));

        assert_ne!(map.get(0, 0), None);
        assert_eq!(map.get(1, 0), Some(0.5));
        assert_ne!(map.get(2, 0), None);
    }

    #[test]
    fn regen() {
        let mut map = HeightMap::new(MIN_SIZE, 0.15, vec![]);

        map.set(1, 0, 0.5);
 
        assert_eq!(map.regen(0, 0), (map.get(0, 0), map.get(0, 0)));
        assert_ne!(map.regen(1, 0), (map.get(1, 0), None));
        assert_ne!(map.regen(2, 0), (map.get(2, 0), None));

        assert_ne!(map.get(0, 0), None);
        assert_ne!(map.get(1, 0), None);
        assert_ne!(map.get(2, 0), None);
    }
}
