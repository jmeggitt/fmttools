# Fmttools
Tools for efficient modification of text as part of a single `write!` call.
 - **No allocation is performed**
 - **Implemented using only safe Rust**

## Examples
### Joining iterator elements
```rust
use fmttools::join;

let elements = vec!["abc", "\n", "123"];
assert_eq!("abc, \n, 123", format!("{}", join(&elements, ", ")));
assert_eq!("\"abc\", \"\\n\", \"123\"", format!("{:?}", join(&elements, ", ")));
```

### Join elements with custom formatting
```rust
use fmttools::join_fmt;

// Alternatively, a closure can be used
fn format_element(x: &i32, f: &mut Formatter<'_>) -> fmt::Result {
    if *x > 3 {
        return write!(f, "3+");
    }

    write!(f, "{}", x)
}

let elements = vec![1, 2, 3, 4, 5];
assert_eq!("1, 2, 3, 3+, 3+", format!("{}", join_fmt(&elements, ", ", format_element)));
```

## Replace arbitrary patterns
```rust
use fmttools::replace;

#[derive(Debug)]
struct FooBar {
    a: String,
}

let value = FooBar { a: "Bar".to_string() };
assert_eq!("FooBiz { a: \"Biz\" }", format!("{:?}", replace(&value, "Bar", "Biz")));
```

## License
Licensed under the Apache License, Version 2.0 https://www.apache.org/licenses/LICENSE-2.0 or the MIT license
https://opensource.org/licenses/MIT, at your option. This file may not be copied, modified, or distributed except
according to those terms.
