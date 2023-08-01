use lazy_diamond_square as lds;
use lds::InitBy as By;
use lds::Parameters::*;

use lds::HeightMap;

use lds_simple_view::gen_img;

fn main() {
    let mut map = HeightMap::new(
        1025,
        0.15,
        vec![
            Seed("view.png"),
            InitLevel(3),
            InitBy(By::Seed),
            CalcRoughnessFn(Box::new(|_x, _y, r| r)),
            ChangeCalcedHeightFn(Box::new(|_x, _y, h| h)),
        ],
    );

    map.gen_all();

    gen_img(&map, "view.png");
}
