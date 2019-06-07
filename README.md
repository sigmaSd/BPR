# BPR
Brute force rust Pub Remover

## What is this???
BPR will check you're rust project for needless pub keywords

To be used on binary porjects (doesn't make sense on a lib, you have to control the API)

## Actual example
Here is IRust diff after using BPR on it  [pub_diff](https://github.com/sigmaSd/IRust/commit/def2c4c824c86e2b5ad510d59ca9fc584b1ca0ac)

## Usage
`cd my_awesome_rust_project` then `bpr`

or

`bpr path_to_my_awesome_project`

this is a dry-run (will only print the needless pub keyword locations without file modification)

If you want to actually remove the needless pub from your project use the `-i` flag

**!!! Be sure to commit your changes before using the '-i' flag !!!**
