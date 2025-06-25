# resTYPErant
Cooking/Typing game playable at https://moveaxesp.github.io/res_type_rant/

This is (going to be) a simple Overcook-like game with typing.  It's also my intro to Rust, so it
should not be used as an example for anything.

## Layout

* *index.html* - top-level HTML of the game
* *Cargo.toml*, *src/* - the Rust source for the core game
* *pkg/* - the compiled wasm of the Rust source
* *images* - images

## Building

To compile the Rust into wasm, run `wasm-pack build --target web` at the root of the repo. Setting up
Rust/Wasmpack is left as an exercise for the user.  This depends on https://github.com/moveaxesp/engine_p,
which must be cloned as 'engine_p' at the same level as this repo.

## Testing

To test locally, run a local web server (`python -m http.server` works) at the root of the repo,
and visit `localhost:8000` (if thats the port) in a browser.