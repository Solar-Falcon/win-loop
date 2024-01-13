# win-loop

Windowing (using `winit`[1]), nice input handling and frame-rate-independent game loop all wrapped up in a neat little package.
The game loop is based on <https://gafferongames.com/post/fix_your_timestep>.

Originally part of [`pix-win-loop`][2], now moved to a separate crate.

The crate features `rwh_04`, `rwh_05` and `rwh_06` enable corresponding `winit` features.
By default `winit` has all its default features enabled except `rwh_06`, so you have to specify one of the `rwh`s in case you need them.

Warning! Crate versions 0.3 and lower might fail to compile on web because of a silly mistake. Should be fixed in 0.4.0.

[1]: https://crates.io/crates/winit
[2]: https://crates.io/crates/pix-win-loop
