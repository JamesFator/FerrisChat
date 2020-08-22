# FerrisChat
Chat in a 2D virtual space where every client is represented with a crab

## Dependencies
```bash
$ cargo install cargo-web
```

## Running
```bash
# Running headless server
$ cargo run --bin server

# Running standalone client
$ cargo web start --bin ferris_chat_client
# Load http://127.0.0.1:8000/
```

## Resources
I knew very little about game development and Rust, so here's a list of resources which I'd recommend.

[Roguelike Tutorial - In Rust](https://bfnightly.bracketproductions.com/rustbook/)

[RustConf 2018 - Closing Keynote - Using Rust For Game Development by Catherine West](https://www.youtube.com/watch?v=aKLntZcp27M)

[tensor-programming/wasm_snake_example](https://github.com/tensor-programming/wasm_snake_example)

[Overwatch Gameplay Architecture and Netcode](https://www.youtube.com/watch?v=W3aieHjyNvw)


## License
[MIT](https://choosealicense.com/licenses/mit/)
