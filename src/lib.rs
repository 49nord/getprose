//! Tools for localizing text, numbers, and dates and times.
//!
//! See [here](https://www.gnu.org/software/gettext/manual/gettext.html#Overview-of-GNU-gettext) for
//! more information on the `gettext` workflow.
//!
//! Translation using gettext looks like this (using [FormatBuilder](FormatBuilder) for formatting):
//!
//! ```rust
//! use getprose::{self, Locale, ToFormat};
//! use gettext::Catalog;
//! use once_cell::sync::OnceCell;
//! use std::collections::HashMap;
//!
//! /// All gettext catalogs, which in turn contain all translations.
//! static CATALOGS: OnceCell<HashMap<Locale, Catalog>> = OnceCell::new();
//!
//! // Initialize `CATALOGS` first.
//!
//! pub fn get_good_text(locale: Locale) -> String {
//!     let localizer = Locale::de_DE.build_localizer(
//!         CATALOGS
//!             .get()
//!             .expect("CATALOGS has to initialized before it can be used"),
//!     );
//!
//!     // Translate a singular string.
//!     localizer.gettext("the first singular");
//!
//!     // Translate a singular string but give some context to be considered when translating.
//!     localizer.pgettext("good_text_context", "the second singular string");
//!
//!     // Translate a string depending on how many `n` there are.
//!     let n = 20;
//!     localizer.ngettext("one string", "{count} strings", n) // Still contains `{count}`.
//!         .to_format() // Convert the &str to a FormatBuilder
//!         .arg("count", &getprose::format_int(n, locale)) // Localize `n` to fill `{count}`
//!         .format();
//!
//!     // Translate a string depending on how many `n` there are, but give some context to
//!     // be considered when translating.
//!     localizer.npgettext("good_text_context", "one string", "{count} strings", n)
//!         .to_format() // Convert the &str to a FormatBuilder
//!         .arg("count", &getprose::format_int(n, locale)) // Localize `n` to fill `{count}`
//!         .format()
//! }
//! ```

use dynfmt::curly::SimpleCurlyFormat;
use dynfmt::{Error as DynFmtError, Format};
use format_num::format_num;
use gettext::Catalog;
use num_format::ToFormattedString;
use std::borrow;
use std::collections::HashMap;
use thiserror::Error;

/// The supported locales and central part of the localization.
///
/// See module-level documentation for more information on how to use this to localize strings.
///
/// # Panics
///
/// Panics if any of the `gettext` methods are called before [init_localization].
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Locale {
    // Do NOT change the variant to number mapping, doing so is a breaking change.
    de_DE = 0,
    en_GB = 1,
    es_ES = 2,
    fr_FR = 3,
    it_IT = 4,
    pt_PT = 5,
    ru_RU = 6,
}

impl<'a> Locale {
    pub fn build_localizer(&self, catalogs: &'a HashMap<Locale, Catalog>) -> Localizer<'a> {
        let catalog = catalogs.get(self).expect(&format!(
            "Could not find translations for locale {:?}",
            self
        ));
        Localizer(catalog)
    }
}

pub struct Localizer<'a>(&'a Catalog);

impl<'a> Localizer<'a> {
    /// Gets a translation for `singular`.
    pub fn gettext(&self, singular: &'static str) -> &'a str {
        self.0.gettext(singular)
    }

    /// Gets a translation either for `singular` or for `plural` depending on `n` and the plural
    /// rules of the locale.
    pub fn ngettext(&self, singular: &'static str, plural: &'static str, n: u64) -> &'a str {
        self.0.ngettext(singular, plural, n)
    }

    /// Gets a translation for `singular`, but provide the translator with a context where this
    /// string is used.
    pub fn pgettext(&self, context: &'static str, singular: &'static str) -> &'a str {
        self.0.pgettext(context, singular)
    }

    /// Gets a translation either for `singular` or for `plural` depending on `n` and the plural
    /// rules of the locale, but provide the translator with a context where this
    /// string is used.
    pub fn npgettext(
        &self,
        context: &'static str,
        singular: &'static str,
        plural: &'static str,
        n: u64,
    ) -> &'a str {
        self.0.npgettext(context, singular, plural, n)
    }
}

impl From<Locale> for num_format::Locale {
    fn from(locale: Locale) -> Self {
        match locale {
            Locale::de_DE => num_format::Locale::de,
            Locale::en_GB => num_format::Locale::en_GB,
            Locale::es_ES => num_format::Locale::es,
            Locale::fr_FR => num_format::Locale::fr,
            Locale::it_IT => num_format::Locale::it,
            Locale::pt_PT => num_format::Locale::pt,
            Locale::ru_RU => num_format::Locale::ru,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<Locale> for chrono::Locale {
    fn from(locale: Locale) -> Self {
        match locale {
            Locale::de_DE => chrono::Locale::de_DE,
            Locale::en_GB => chrono::Locale::en_GB,
            Locale::es_ES => chrono::Locale::es_ES,
            Locale::fr_FR => chrono::Locale::fr_FR,
            Locale::it_IT => chrono::Locale::it_IT,
            Locale::pt_PT => chrono::Locale::pt_PT,
            Locale::ru_RU => chrono::Locale::ru_RU,
        }
    }
}

impl std::str::FromStr for Locale {
    type Err = UnknownLocaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de_DE" | "de" => Ok(Locale::de_DE),
            "en_GB" | "en" => Ok(Locale::en_GB),
            "es_ES" | "es" => Ok(Locale::es_ES),
            "fr_FR" | "fr" => Ok(Locale::fr_FR),
            "it_IT" | "it" => Ok(Locale::it_IT),
            "pt_PT" | "pt" => Ok(Locale::pt_PT),
            "ru_RU" | "ru" => Ok(Locale::ru_RU),
            _ => Err(UnknownLocaleError(s.to_owned())),
        }
    }
}

/// Received an unknown locale.
#[derive(Debug, Error, Clone)]
#[error("Unknown locale {0}")]
pub struct UnknownLocaleError(String);

/// Format `&str` during runtime.
pub struct FormatBuilder<'a> {
    /// The template defining the formatting.
    tpl: &'a str,
    /// The arguments used in formatting.
    args: HashMap<&'a str, String>,
}

impl<'a> FormatBuilder<'a> {
    /// Adds an argument to be used in formatting.
    pub fn arg<S: ToString>(&mut self, key: &'a str, value: &S) -> &mut Self {
        self.args.insert(key, value.to_string());
        self
    }

    /// Adds all arguments contained in `args` to `self.args`.
    pub fn args<S: ToString>(&mut self, args: HashMap<&'a str, S>) -> &mut Self {
        self.args
            .extend(args.iter().map(|(&k, v)| (k, v.to_string())));
        self
    }

    /// Formats the given template with the added args with [try_format](FormatBuilder::try_format) if possible.
    /// If not, the template will be returned as is.
    pub fn format(&self) -> String {
        // Try to format `self.templ`, but fallback to no formatting if `self.args` is missing an argument.
        self.try_format()
            .unwrap_or_else(|_| self.noop_format())
            .to_string()
    }

    /// Formats the given template and returns an error if it failed.
    pub fn try_format(&self) -> Result<borrow::Cow<'a, str>, DynFmtError> {
        SimpleCurlyFormat.format(self.tpl, &self.args)
    }

    /// Returns the template as is.
    fn noop_format(&self) -> borrow::Cow<'a, str> {
        // This should never fail to format, since NoopFormat is being used
        dynfmt::NoopFormat.format(self.tpl, &self.args).unwrap()
    }
}

/// A trait to help with creating a [FormatBuilder](FormatBuilder).
pub trait ToFormat {
    /// Create a `FormatBuilder` from `&self`.
    fn to_format(&self) -> FormatBuilder;
}

impl<'a> ToFormat for &'a str {
    fn to_format(&self) -> FormatBuilder {
        FormatBuilder {
            tpl: self,
            args: HashMap::new(),
        }
    }
}

/// Formats `n` according to `locale`.
pub fn format_int<N: num_format::ToFormattedStr>(n: N, locale: Locale) -> String {
    n.to_formatted_string::<num_format::Locale>(&locale.into())
}

/// Formats `f` as an `f64` with `precision` digits after the decimal point according to `locale`.
///
/// If necessary `f` is rounded to `precision` by rounding halves away from zero.
pub fn format_f64<N: Into<f64>>(f: N, precision: u8, locale: Locale) -> String {
    let nf_locale: num_format::Locale = locale.into();
    format_num!(&format!(",.{}f", precision), f)
        .replace(',', "!")
        .replace('.', nf_locale.decimal())
        .replace('!', nf_locale.separator())
}

#[cfg(test)]
mod tests {
    use super::{format_f64, Locale};

    #[test]
    fn format() {
        assert_eq!(&format_f64(0, 0, Locale::de_DE), "0");
        for precision in 1..10 {
            assert_eq!(
                format_f64(0, precision, Locale::de_DE),
                "0".to_string() + "," + &"0".repeat(precision as usize)
            );
        }

        assert_eq!(&format_f64(0.0, 0, Locale::de_DE), "0");
        assert_eq!(&format_f64(0.0000000000000001, 0, Locale::de_DE), "0");
        assert_eq!(
            &format_f64(0.0000000000000001, 16, Locale::de_DE),
            "0,0000000000000001"
        );
        assert_eq!(&format_f64(0.001, 2, Locale::de_DE), "0,00");
        assert_eq!(&format_f64(0.005, 2, Locale::de_DE), "0,01");
        assert_eq!(&format_f64(0.009, 2, Locale::de_DE), "0,01");
        assert_eq!(&format_f64(1.1234, 3, Locale::de_DE), "1,123");
        assert_eq!(&format_f64(1.1234, 4, Locale::de_DE), "1,1234");
        assert_eq!(&format_f64(1.1234, 5, Locale::de_DE), "1,12340");

        assert_eq!(&format_f64(-0.0, 0, Locale::de_DE), "0");
        assert_eq!(&format_f64(-0.0000000000000001, 0, Locale::de_DE), "0");
        assert_eq!(
            &format_f64(-0.0000000000000001, 16, Locale::de_DE),
            "-0,0000000000000001"
        );
        assert_eq!(&format_f64(-0.001, 2, Locale::de_DE), "0,00");
        assert_eq!(&format_f64(-0.005, 2, Locale::de_DE), "-0,01");
        assert_eq!(&format_f64(-0.009, 2, Locale::de_DE), "-0,01");
        assert_eq!(&format_f64(-1.1234, 3, Locale::de_DE), "-1,123");
        assert_eq!(&format_f64(-1.1234, 4, Locale::de_DE), "-1,1234");
        assert_eq!(&format_f64(-1.1234, 5, Locale::de_DE), "-1,12340");

        assert_eq!(&format_f64(1234, 5, Locale::de_DE), "1.234,00000");
        assert_eq!(&format_f64(-1234, 5, Locale::de_DE), "-1.234,00000");
    }
}
