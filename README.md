# win-loop

Windowing (using `winit`[1]), nice input handling and frame-rate-independent game loop all wrapped up in a neat little package.
The game loop is based on <https://gafferongames.com/post/fix_your_timestep>.

Originally part of [`pix-win-loop`][2], now moved to a separate crate.

## Cargo features

The crate features `rwh_04`, `rwh_05` and `rwh_06` enable corresponding `winit` features.
By default `winit` has all its default features enabled except `rwh_06`, so you have to specify one of the `rwh`s in case you need them.

Note:
As of version 0.6 and further (see note 2 for caveat), all of `winit`'s features are disabled.
You can still enable `rwh_NN` directly from this crate's features. If you need to enable others, add something like:

```toml
[dependencies]
# ...
winit = { version = "0.29", features = [ ...whatever you need... ] }
```

to your `Cargo.toml`.

Note 2:
In version 0.6.1, the `winit-default` feature has been added, which enables `winit`'s default features, except for `rwh_06`.
It is enabled by default in 0.6.1 so that `docs.rs` can actually build the crate.

In 0.6.2 and further it is disabled again in favour of using `[package.metadata.docs.rs]` which I discovered like 3 minutes after publishing 0.6.1. Sorry for the mess.
0.6.0, 0.6.1 and 0.6.2 are fully interchangeable unless you turn off `default-features`, so there shouldn't be many problems from all this.

## Warning

Crate versions 0.3 and lower might fail to compile on web because of a silly mistake. Should be fixed in 0.4.0.

[1]: https://crates.io/crates/winit
[2]: https://crates.io/crates/pix-win-loop
