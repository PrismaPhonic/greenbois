# Greenbois

This application allows you to fill out green squares on github by creating thousands of fake commits dating
from as far back as your heart desires. Defaults to starting 1 year in the past.

## Installing

To install this, you first need to [install cargo](https://rustup.rs/).

Once you have cargo installed you can install `greenbois` globally by typing into your terminal from the root directory:

```sh
$ cargo install --path .
```

## Use

1. Create a private repo on github.
2. Create a folder locally to match this new repo.
3. Run `git init` inside the folder.
4. Run `greenbois` using the `commit` command, pass it a message with the `-m` flag (for now this message,
will apply to all commits made, but that doesn't matter.) and `-y` flag to specify a number of years in the past to create the init commit.
Commits will be created from this date, skipping major US holidays and weekends, and randomizing the # of commits made per day. You may also 
optionally supply a start and end time for your work day in hours (24 hour
format) with the `-s` and `-e` flags respectively.

Full example command: 

```sh
$ greenbois commit -m "Stupid message." -y 1.5 -s 9 -e 17
```

Have fun!
