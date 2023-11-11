# incro
**IN**put ma**CRO**s: program your keyboard/mouse

See `macro-template` for macro example. Build it and place the resulting `*.so` file in `macros/` directory to test it.

## Running

```sh
cargo run -r -p incro-bin
```

## Workflow

I recommend copying the template into `macros/` directory and making your macros there, symlink their built dynamic libraries to the same `macros/` directory and now you can just rebuild the workspace with this:

```sh
cargo build -r --workspace
```

and that will update all your macros.
