# Lazy Diamond-Square

The lazy_diamond_square(hereafter LDS) will allow you to use the Diamond-Square algorithm to generate your own landscapes or anything else where it might come in handy.

Here is what I have implemented in version 0.1.0 and plan to implement in future versions:
* The ability to reproduce the result, i.e. not using time-dependent pseudo-random numbers on your device, when calculating the height of a point. +

* Ability to not generate the whole map at once, but only some parts of it that are needed. +

* Maximum optimization of the project to work even on weak machines. + -
 - Optimize everything that can be optimized without any fancy actions (for example, add ´if´ operator not at the beginning of the loop, but before it, if possible). +
 - Add optimized division of the number modulo. -
 - Add multithreading and asynchronous execution. -

* Modify functions with the postfix ´_all´ so that they do not apply the method to every point on the map, but only to the specified area, and replace the postfix with ´_area´. -

* Add more information about the project to the README file and make the text inside the file more readable. -

# Sources I took inspiration and information from

* [Habr article](https://habr.com/ru/articles/111538/) (it's in Russian, so I suggest you use an extension in your browser, like ImTranslator for FireFox, for example).

* [Wikipedia page (versions in all languages except the one with ligatures)](https://en.wikipedia.org/wiki/Diamond-square_algorithm)

* [JS implementation](https://github.com/hunterloftis/playfuljs-demos/blob/gh-pages/terrain/index.html)

* [Python implementation](https://github.com/buckinha/DiamondSquare/tree/master)

* And anything else you can google for "diamond square". It's very long to list everything, so I just pointed out the main things and this item.

# Explanation of how the project works

There is nothing here yet, but in future versions I will add here a detailed explanation of how it works. For now, you only need to look at the project documentation and you will learn the essentials.
I'm going to reinstall my OS the other day, so I'm posting the project in a hurry and raw. To be more precise, only the README file is not completely finished.

# Example of work

![Example](/view.png "Example")


# !!!

I just wanted to let you know that I'm 13 and this is my first serious project, so I'll be only glad for criticism and advice on how to improve my project. All text in this file is translated with the help of a translator, because I don't know English so well yet.