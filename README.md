# pix-win-loop

GPU pixel buffer (using [`pixels`][1]), windowing (using [`winit`][2]), nice input handling and frame-rate-independent game loop all wrapped up in a neat little package.
The game loop is based on <https://gafferongames.com/post/fix_your_timestep>.

Originally part of [`pix-win-loop`][3], now moved to a separate crate.

The crate features `rwh_04`, `rwh_05` and `rwh_06` enable corresponding `winit` features.
By default `winit` has all its default features enabled except `rwh_06`, so you have to specify one of the `rwh`s in case you need them.

[1]: https://crates.io/crates/pixels
[2]: https://crates.io/crates/winit
[3]: https://crates.io/crates/pix-win-loop
