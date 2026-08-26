# tidyunits

Byte sizes and durations show up as free text almost everywhere: config
files, CLI flags, log lines, a value someone typed into a form. The same
ninety seconds might arrive as `90s`, `1.5m`, `1:30`, or `00:01:30`, and the
same ten megabytes might arrive as `10MB`, `10 mb`, or `10,000,000`. Before
you can compare, store, or bucket these values you need one canonical
number.

tidyunits parses that mess into a plain `u64` byte count or a
`std::time::Duration`, and can format a value back into a short, consistent
string.

## Usage

Add it as a path dependency (not published yet):

```toml
[dependencies]
tidyunits = { path = "../tidyunits" }
```

```rust
use tidyunits::{format_bytes, format_duration, parse_bytes, parse_duration};

let a = parse_bytes("1.5 MB").unwrap();
let b = parse_bytes("1,5MB").unwrap();
assert_eq!(a, b);
assert_eq!(format_bytes(a), "1.43 MiB");

let d1 = parse_duration("1h30m").unwrap();
let d2 = parse_duration("90m").unwrap();
let d3 = parse_duration("1:30:00").unwrap();
assert_eq!(d1, d2);
assert_eq!(d1, d3);
assert_eq!(format_duration(d1), "1h30m0s");
```

## What it normalizes

- **Byte sizes** — decimal units (`kb`, `mb`, `gb`, `tb`, base 1000) and
  binary units (`kib`, `mib`, `gib`, `tib`, base 1024), case-insensitive,
  with or without a space before the unit, a comma or dot as the decimal
  separator, underscores as digit separators, and a bare number read as
  bytes.
- **Durations** — compound unit strings (`1h30m`, `1d2h`), a single unit
  with a decimal value (`1.5h`), colon clock notation (`1:30:00`, `5:09`),
  and a bare number read as seconds.

Only the failure modes and edge cases actually tested are guaranteed right
now — see the test tables at the bottom of `src/bytes.rs` and
`src/duration.rs`.

## Status

Early skeleton. Parsing and formatting for both units work and are covered
by table-driven tests, but the unit list is intentionally small (no bits,
no weeks) until something needs them.

## License

MIT
