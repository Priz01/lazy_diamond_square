# Lazy Diamond-Square

The lazy_diamond_square(hereafter LDS) will allow you to use the Diamond-Square algorithm to generate your own landscapes or anything else where it might come in handy.

## Example

```rust
use lazy_diamond_square as lds;
use lds::InitBy as By;
use lds::Parameters::*;

use lds::HeightMap;

use lds_simple_view::gen_img;

fn main() {
    let mut map = HeightMap::new(
        513,
        0.15,
        vec![
            Seed("view.png"),
            InitLevel(2),
            InitBy(By::Seed),
        ],
    );

    map.gen_all();

    gen_img(&map, "view.png");
}
```

## Examples of work

roughness = 0.15, Seed("view1.png")
![Example 1: roughness = 0.15, Seed("view1.png")](/view1.png)

roughness = 0.2, Seed("view2.png")
![Example 2: roughness = 0.2, Seed("view2.png")](/view2.png)

roughness = 0.2, Seed("view3.png"), InitLevel(4)
![Example 3: roughness = 0.2, Seed("view3.png"), InitLevel(4)](/view3.png)

## How it works

There's nothing here yet, but I'll add something here in future versions. For now, just take a look at the project documentation and you'll know the essentials.
I'm leaving tomorrow to visit my grandmother in the country, so these changes are minor.
To explain how LDS works, I plan to post an article on Habr, but if there are enough people interested, I will describe how LDS works here too, in English. Or I'll post the article somewhere.

## TODO

- [x] The ability to reproduce the result, i.e. not using time-dependent pseudo-random numbers on your device, when calculating the height of a point.

- [x] Ability to not generate the whole map at once, but only some parts of it that are needed.

- [ ] Maximum optimization of the project to work even on weak machines.
    - [x] Optimize everything that can be optimized without any fancy actions (for example, add ´if´ operator not at the beginning of the loop, but before it, if possible).
    - [ ] Add optimized division of the number modulo.
    - [ ] Add multithreading and asynchronous execution.

- [ ] Modify functions with the postfix ´_all´ so that they do not apply the method to every point on the map, but only to the specified area, and replace the postfix with ´_area´.

- [ ] Add more information about the project to the README file.

## !!!

I just wanted to let you know that I'm 13 and this is my first serious project, so I'll be only glad for criticism and advice on how to improve my project. All text in this file is translated with the help of a translator, because I don't know English so well yet.

## Sources

* [Habr article](https://habr.com/ru/articles/111538/) (it's in Russian, so I suggest you use an extension in your browser, like ImTranslator for FireFox, for example).

* [Wikipedia page (versions in all languages except the one with ligatures)](https://en.wikipedia.org/wiki/Diamond-square_algorithm)

* [JS implementation](https://github.com/hunterloftis/playfuljs-demos/blob/gh-pages/terrain/index.html)

* [Python implementation](https://github.com/buckinha/DiamondSquare/tree/master)

* And anything else you can google for "diamond square". It's very long to list everything, so I just pointed out the main things and this item.

## Question for you

Do I need to describe the changes to the current version here, or is a brief explanation of the commit on GitHub sufficient? Write your answer to my mail, [you know where to find it](https://doc.rust-lang.org/cargo/reference/manifest.html#the-authors-field).