# Risto

Cleaning up your music is a pain, don't want to open every single file and most players I know of don't have delete button for current song.

Risto is a poor man's music rater, it will keep asking while playing songsuntil files in folder are exhausted.

![Example](./.readme/example.png "Example")

Results are written to a json file, later on you can process that and remove songs you didn't like or move them elsewhere.

You could classify and remove files you marked as not liked as follows.

```nushell
# download a release execute: `risto -- ~/Music/`
# or build and run it as follows
cargo run --bin risto -- ~/Music/
open likes.json | filter {$in.like == "No" } | each { rm --trash $in.path}
```
