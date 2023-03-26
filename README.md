# tilos

my very own puzzle solver.

## what can it do?

it can solve all 35 in-game puzzles extremely quickly (the slowest taking roughly a tenth of a second), which is super cool. here's a visual example of what solving an extremely simple puzzle looks like:

> board:
>
> ⬛️⬛️⬛️⬛️\
> ⬛️⬛️⬛️⬛️
>
> pieces:
>
> 🟥⬛️⬛️\
> 🟥🟥🟥
>
> 🟦⬛️⬛️\
> 🟦🟦🟦
>
> possible solution:
>
> 🟥🟦🟦🟦\
> 🟥🟥🟥🟦

## how can i try it?

there's an `.html` page inside the `www/` directory. you can download that and open it in your web-browser of choice. by default you'll find the puzzle i was stuck on, and you can try to solve it by hand, or create your own puzzle configurations.

the web version also includes all puzzles from the game (i got those from a [steam guide with all the solutions][puzzle-solutions-guide], and i just wrote the board sizes and keys used for each). however it doesn't feel quite the same to use someone else's solution, so i still prefer to have my own code being able to solve the puzzles. looking up all puzzle solutions was just the easiest way to grab all in-game puzzles.

[puzzle-solutions-guide]: https://steamcommunity.com/sharedfiles/filedetails/?id=354590899

## history

i finally got to play "the talos principle", which i bought many years back. i do eventually give up on some of the puzzles when my brain can't think. but so far, the little tetris-like puzzle were not posing a challenge. until one did. and here we are!

i could not be bothered to think of a solution, so i decided to program it instead. as you do.

the name comes from the pieces ("sigils") which are available to you to solve the little puzzles. a t-shape, i-shape, l-shape, o-shape (squares are close enough to circles, right?), and s-shape (you could argue it's more of a "z", but who cares). tilos.

when i realized the game was called "talos", the pieces for naming my little helper "tilos" just kind of fell into place (see what i did there?).

## license

listen, i am not a lawyer. but i like rust and rust's decision to dual-license stuff under mit and apache is fine by me. so the files in this project are dual-licensed under those terms, too.
