# incro
**IN**put ma**CRO**s: program your keyboard/mouse

See `macro-template` for macro example. Build it and place the resulting `*.so` file in `macros/` directory to test it.

## Running

```sh
cargo run -r -p incro-bin
```

Your user must have access to the files in `/dev/input/`, so either root or in a group of the file. Run

```sh
ls -la /dev/input
```

to see what group has access to them.

## Workflow

I recommend copying the template into `macros/` directory and making your macros there, symlink their built dynamic libraries to the same `macros/` directory and now you can just rebuild the workspace with this:

```sh
cargo build -r --workspace
```

and that will update all your macros.
