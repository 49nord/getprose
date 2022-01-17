# getprose

Tools for localizing text, numbers, and dates and times.

See [here](https://www.gnu.org/software/gettext/manual/gettext.html#Overview-of-GNU-gettext) for
more information on the `gettext` workflow.

Translation using gettext looks like this:

```rust
use getprose::{self, Locale, ToFormat};
use gettext::Catalog;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/// All gettext catalogs, which in turn contain all translations.
static CATALOGS: OnceCell<HashMap<Locale, Catalog>> = OnceCell::new();

// Initialize `CATALOGS` first.

pub fn get_good_text(locale: Locale) -> String {
    let catalog = Locale::de_DE.get_catalog(
        CATALOGS
            .get()
            .expect("CATALOGS has to initialized before it can be used"),
    );

    // Translate a singular string.
    catalog.gettext("the first singular");

    // Translate a singular string but give some context to be considered when translating.
    catalog.pgettext("good_text_context", "the second singular string");

    // Translate a string depending on how many `n` there are.
    let n = 20;
    catalog.ngettext("one string", "{count} strings", n) // Still contains `{count}`.
        .to_format() // Convert the &str to a FormatBuilder
        .arg("count", &getprose::format_int(n, locale)) // Localize `n` to fill `{count}`
        .format();

    // Translate a string depending on how many `n` there are, but give some context to
    // be considered when translating.
    catalog.npgettext("good_text_context", "one string", "{count} strings", n)
        .to_format() // Convert the &str to a FormatBuilder
        .arg("count", &getprose::format_int(n, locale)) // Localize `n` to fill `{count}`
        .format()
}
```

# Features

- `chrono`: implements `From<getprose::Locale>` for `chrono::Locale`.
