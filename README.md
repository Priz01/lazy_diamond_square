# Lazy Diamond-Square

The lazy_diamond_square (hereafter LDS) will allow you to use the Diamond-Square algorithm to generate your own landscapes or anything else where it might come in handy.

## Example

```rust
use lazy_diamond_square as lds;
use lds::{Builder, InitBy as By};

fn main() {
    let mut map = Builder::new(513, 0.15)
        .seed("qwerty")
        .init_lvl(2)
        .init_by(By::Seed)
        .build();
    let max_coord = map.max_coord();

    map.gen_area( (0, 0), (max_coord, max_coord));
    map.get_img((0, 0), (max_coord, max_coord), Some("view.png"));
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

- [x] Modify functions with the postfix ´_all´ so that they do not apply the method to every point on the map, but only to the specified area, and replace the postfix with ´_area´.

- [ ] Add more information about the project to the README file.

## !!!

I just wanted to let you know that I'm 13 and this is my first serious project, so I'll be only glad for criticism and advice on how to improve my project. All text in this file is translated with the help of a translator, because I don't know English so well yet.

## Sources

* [Habr article](https://habr.com/ru/articles/111538/) (it's in Russian, so I suggest you use an extension in your browser, like ImTranslator for FireFox, for example).

* [Wikipedia page (versions in all languages except the one with ligatures)](https://en.wikipedia.org/wiki/Diamond-square_algorithm)

* [JS implementation](https://github.com/hunterloftis/playfuljs-demos/blob/gh-pages/terrain/index.html)

* [Python implementation](https://github.com/buckinha/DiamondSquare/tree/master)

* And anything else you can google for "diamond square". It's very long to list everything, so I just pointed out the main things and this item.