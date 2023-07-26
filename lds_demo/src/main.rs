use lazy_diamond_square as lds;
use lds::InitBy as By;
use lds::Parameters::*;

use lds::HeightMap;

use lds_simple_view as lsv;

use lsv::gen_img;

fn main() {
    let mut map = HeightMap::new(
        1025,
        0.15,
        vec![
            Seed("qwerty"),
            InitLevel(3),
            InitBy(By::Seed),
            CalcRoughnessFn(Box::new(|_x, _y, r| r)),
            ChangeCalcedHeightFn(Box::new(|_x, _y, h| h)),
        ],
    );

    map.gen_all();

    gen_img(map, Box::new(|x, y| (x, y)), 1);
}
