# Risto

Cleaning up your music is a pain, don't want to open every single file and most players I know of luckily don't have a button to delete current song.

Risto is a poor man's music rater it helps you classify your music in two categories, the songs you like and the ones you don't, no more no less.

Results are written to a json file, later on you can process that and remove songs you didn't like or move them elsewhere.

You could classify and remove files you marked as not liked as follows.

```nushell
cargo run ~/Music
open likes.json | filter {$in.like == "No" } | each { rm --trash $in.path}   
```
