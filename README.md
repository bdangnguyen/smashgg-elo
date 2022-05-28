# smashgg-elo

A reimplementation of an earlier program written in Python using Rust for the purpose of learning the language. This program queries the api for https://smash.gg tournament data, parses it, calculates and ranks players using the [Elo rating system](https://en.wikipedia.org/wiki/Elo_rating_system), and records all data into a local sqlite database.

Using [DB Browser](https://sqlitebrowser.org/), this is what the sqlite database looks like after parsing through a few local tournaments
![Demo](https://user-images.githubusercontent.com/31323380/166092573-3e8dd0e9-3648-4951-915d-af601b055562.jpg)

## Usage
If you prefer to use only the .exe, download the binary [here](https://github.com/bdangnguyen/smashgg-elo/releases/tag/v1.0.1). Then simply run it using
```
smashgg_elo.exe
```

You will need the following for this program to work correctly:
* A smash.gg authentication token which you can learn on how to generate from [smash.gg's developer page](https://developer.smash.gg/docs/authentication).
* The tournament slug for the tournament. A slug is a specific part of the url for a smash.gg tournament. More specifically, you want to write down
```
https://smash.gg/tournament/[tournament slug]/...
```

Follow the prompts in the command line terminal to generate a sqlite database containing the Elo calculations. There is a general table for an overall Elo calculation called *players*, a table that records the results of a set in a tournament called *sets*, and a table for each respective game that was parsed. 

## Building
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone or fork this repo and `cd` to it
3. Use `cargo run`

## License
This project is licensed under the [MIT license](https://github.com/bdangnguyen/smashgg-elo/blob/main/LICENSE)
