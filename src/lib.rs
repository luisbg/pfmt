/*!
 * # Overview
 * This library provides a flexible and powerful method to format data. At the
 * heart of the library lie two traits: `Fmt`, for things that are formattable,
 * and `FormatTable`, that maps placeholders in format strings to actual `Fmt`s
 * and supply those with information they need to produce output. Unlike with
 * `format!` from the standard library, there is no restriction that format
 * strings need to be static; in fact the whole point of the library is to
 * allow moving as much control over formatting process into the format strings
 * themselves (and ideally those - in user-editable config files).
 *
 * There are several `impl`s of `FormatTable`, most notable for `HashMap`s
 * (with either `str` or `String` keys, and `Borrow<Fmt>` values, which means a
 * bit of type annotations required to use those) and `Vec`s (with
 * `Borrow<Fmt>` elements). The method on `FormatTable` to format a string is
 * `format(&self, format_string: &str) -> Result<String, FormattingError>`
 *
 * Each format string consists of one or several literals and placeholders,
 * optionally separated by colons ("`:`"). If you need a colon in your literal
 * or some part of a placeholder, you need to escape it: `"\:"`. In its
 * simplest form, a placeholder looks like this: `"{foobar}"` (brackets can be
 * escaped too, if you need them in a literal or somewhere else). This will
 * request the format table that does the formatting to lookup a `Fmt` named
 * "foobar" and insert it contents there, or fail if it cannot find it (or
 * produce it - see 'More fun' section). Of course, this alone is not much use.
 * Most `Fmt`s support flags that can change the output of the formatting
 * procedure, for instance floats can request to be printed in exponential
 * notation. Flags are just single characters and are separated from the
 * identifier with a colon: `"{foobar:flags}"`. A trailing colon is allowed.
 * Some `Fmt`s also support options, which are specified after the flags (and
 * if you want to use options, you need a flags section, even if it's empty)
 * and are separated by colons: `"{foobar::option1=value1:option2=value2}"`.
 * There aren't too many options at the moment. There is also a possibility of
 * giving arguments to a placeholder, but there's no implementation (yet) of a
 * `FormatTable` that takes advantage of it.
 *
 * See each implementation's entry to learn all the options and flags it
 * supports.
 *
 * So let's see some examples how this all is tied together.
 *
 * # Examples
 * Let's start with something boring:
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable};
 *
 * let i = 2;
 * let j = 5;
 * let mut table: HashMap<&str, &Fmt> = HashMap::new();
 * table.insert("i", &i);
 * table.insert("j", &j);
 * let s = table.format("i = {i}, j = {j}").unwrap();
 * assert_eq!(s, "i = 2, j = 5");
 * ```
 * I can do that with `format!` too. This is a bit more fun, and shows both
 * options and flags:
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable};
 *
 * let s = "a_really_long_string";
 * let i = 10;
 * let j = 12;
 * let mut table: HashMap<&str, &Fmt> = HashMap::new();
 * table.insert("s", &s);
 * table.insert("i", &i);
 * table.insert("j", &j);
 * // (note escaped colons)
 * let s = table.format("hex\\: {i:px}, octal\\: {j:o}, fixed width\\: {s::truncate=r5}").unwrap();
 * assert_eq!(s, "hex: 0xa, octal: 14, fixed width: a_rea");
 * ```
 * Can't decide if you want your booleans as "true" and "false", or "yes" and
 * "no"? Easy:
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable};
 *
 * let a = true;
 * let b = false;
 * let mut table: HashMap<&str, &Fmt> = HashMap::new();
 * table.insert("a", &a);
 * table.insert("b", &b);
 * let s = table.format("{a}, {b:y}, {b:Y}").unwrap();
 * assert_eq!(s, "true, no, N");
 * ```
 * And here are `Vec`s as format tables:
 * ```
 * use pfmt::{Fmt, FormatTable};
 * let i = 1;
 * let j = 2;
 * let table: Vec<&Fmt> = vec![&i, &j];
 * let s = table.format("{0}, {1}, {0}").unwrap();
 * assert_eq!(s, "1, 2, 1");
 * ```
 * All of the above examples used references as the element type of the format
 * tables, but `FormatTable` is implemented (for hashmaps and vectors) for
 * anything that is `Borrow<Fmt>`, which means boxes, and reference counters
 * and more. Tables can fully own the data:
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable};
 *
 * let mut table: HashMap<String, Box<Fmt>> = HashMap::new();
 * table.insert("a".to_string(), Box::new(2) as Box<Fmt>);
 * table.insert("b".to_string(), Box::new("foobar".to_string()) as Box<Fmt>);
 * let s = table.format("{a}, {b}").unwrap();
 * assert_eq!(s, "2, foobar");
 * ```
 * This is a bit on the verbose side, though.
 *
 * The library also suppports accessing elements of `Fmt`s through the same
 * syntax Rust uses: dot-notation, provided the implementation of `Fmt` in
 * question allows it:
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable, SingleFmtError, util};
 * 
 * struct Point {
 *     x: i32,
 *     y: i32
 * }
 * 
 * impl Fmt for Point {
 *     fn format(
 *         &self,
 *         full_name: &[String],
 *         name: &[String],
 *         args: &[String],
 *         flags: &[char],
 *         options: &HashMap<String, String>,
 *     ) -> Result<String, SingleFmtError> {
 *         if name.is_empty() {
 *             Err(SingleFmtError::NamespaceOnlyFmt(util::join_name(full_name)))
 *         } else if name[0] == "x" {
 *             self.x.format(full_name, &name[1..], args, flags, options)
 *         } else if name[0] == "y" {
 *             self.y.format(full_name, &name[1..], args, flags, options)
 *         } else {
 *             Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)))
 *         }
 *     }
 * }
 * 
 * let p = Point { x: 1, y: 2 };
 * let mut table: HashMap<&str, &Fmt> = HashMap::new();
 * table.insert("p", &p);
 * let s = table.format("{p.x}, {p.y}").unwrap();
 * assert_eq!(s, "1, 2");
 * ```
 * This can be nested to arbitrary depth.
 *
 * # Errors
 * `format` method on `FormatTables` returns a `Result<String,
 * FormattingError>`. There are three primary types of these: parsing errors
 * which occur when the format string is not well-formed, errors arising from
 * usage of unknown options and flags or options with invalid values, and
 * finally errors due to requesting `Fmt`s that are missing in the table.
 *
 * With hard-coded format strings and rigid format tables, most of these can be
 * safely ignored, so `unwrap()` away.
 *
 * # Common options
 * Most pre-made implementation of `Fmt` honor several common options. Here's
 * a list of them, with detailed info available further in this section:
 * * `truncate`
 * * `width`
 *
 * ## `truncate`: `{'l', 'r'} + non-negative integer`
 * Controls truncation of the field. If begins with `l`, left part of the
 * field that doesn't fit is truncated, if begins with `r` - the right part is
 * removed instead. Note that `"l0"` is not actually forbidden, just very
 * useless.
 *
 * It is an `InvalidOptionValue` to pass anything not fitting into the template
 * in the header as the value of this option.
 *
 * ## `width`: `{'l', 'c', 'r'} + non-negative integer`
 * Controls the width of the field. Has no effect if the field is already wider
 * than the value supplied. If starts with "`l`", the field will be
 * left-justified. If starts with "`c`", the field will be centered. If starts
 * with "`r`", the field will be right-justified.
 *
 * It is an `InvalidOptionValue` to pass anything not fitting into the template
 * in the header as the value for this option.
 *
 * # Common numeric options
 * Most numeric Fmts honor these. For the detailed description skip to the end
 * of this section.
 * * `prec`
 * * `round`
 *
 * ## `prec`: `integer`
 * Controls precision of the displayed number, with bigger values meaning more
 * significant digits will be displayed. If negative, the number will be
 * rounded, the rounding direction is controlled by the `round` option.
 * Positive values are accepted by integer Fmts, but have no effect.
 *
 * It is an `InvalidOptionValue` to pass a string that doesn't parse as a
 * signed integer as a value to this option.
 *
 * ## `round`: `{"up", "down", "nearest"}`
 * Controls the direction of rounding by the `round` option, and has no effect
 * without it. Defaults to `nearest`.
 *
 * It is an `InvalidOptionValue` to pass a string different from the mentioned
 * three to this option.
 *
 * # More fun
 * Format tables are not required to actually *hold* the `Fmt`s. They can
 * produce those on the fly, if you make them to. You only need to implement
 * the `get_fmt` method a bit differently:
 * ```
 * use pfmt::{Fmt, FormatTable, BoxOrRef};
 *
 * struct Producer { }
 *
 * impl FormatTable for Producer {
 *      fn get_fmt<'a, 'b>(&'a self, name: &'b str)
 *          -> Option<BoxOrRef<'a, dyn Fmt>>
 *      {
 *          if let Ok(i) = name.parse::<i32>() {
 *              Some(BoxOrRef::Boxed(Box::new(i)))
 *          } else {
 *              None
 *          }
 *      }
 * }
 *
 * let table = Producer { };
 * let s = table.format("{1}, {12}").unwrap();
 * assert_eq!(s, "1, 12");
 * ```
 * The above example is not particularly useful, but shows the point.
 *
 * There's also an implementation of `FormatTable` for tuples (up to 6-tuples)
 * that contain format tables. When encountering a placeholder, it first
 * searches for the relevant `Fmt` in the first table, then in the second and
 * so on. This allows to easily override some `Fmt`s or provide defaults
 * without changing the tables themselves.
 * ```
 * use std::collections::HashMap;
 * use pfmt::{Fmt, FormatTable};
 *
 * let i1 = 10;
 * let i2 = 100;
 * let j = 2;
 * let mut table1: HashMap<&str, &Fmt> = HashMap::new();
 * table1.insert("i", &i1);
 * table1.insert("j", &j);
 * let mut table2: HashMap<&str, &Fmt> = HashMap::new();
 * table2.insert("i", &i2);
 * let s = (table2, table1).format("{i}, {j}").unwrap();
 * assert_eq!(s, "100, 2");
 * ```
 */

#[cfg(test)]
#[macro_use]
extern crate galvanic_assert;
#[cfg(test)]
#[macro_use]
extern crate galvanic_test;

extern crate num;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Deref;

use parse::{parse, ParseError, Piece};

mod parse;

pub mod util;

/* ---------- base traits ---------- */

/// This trait drives the formatting of a single placeholder. Placeholder's
/// arguments, flags and options are passed to the `format` method.
pub trait Fmt {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError>;
}

pub trait FormatTable {
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>>;

    fn format(&self, input: &str) -> Result<String, FormattingError> {
        let pieces = parse(input)?;
        let mut res = String::new();
        for piece in pieces.iter() {
            res.push_str(&format_one(self, piece)?);
        }
        Ok(res)
    }
}

fn format_one<'a, 'b, T: FormatTable + ?Sized>(
    table: &'a T,
    piece: &'b Piece,
) -> Result<String, FormattingError> {
    match piece {
        Piece::Literal(s) => Ok(s.clone()),
        Piece::Placeholder(name, args, flags, opts) => {
            if let Some(root) = table.get_fmt(&name[0]) {
                let mut processed_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    processed_args.push(format_one(table, arg)?);
                }
                let mut processed_opts = HashMap::new();
                for (key, piece) in opts.iter() {
                    processed_opts.insert(key.clone(), format_one(table, piece)?);
                }
                Ok(root.format(name, &name[1..], &processed_args, flags, &processed_opts)?)
            } else {
                Err(FormattingError::UnknownFmt(util::join_name(&name)))
            }
        }
    }
}

/* ---------- an important helper thing ---------- */

pub enum BoxOrRef<'a, T: ?Sized + 'a> {
    Boxed(Box<T>),
    Ref(&'a T),
}

impl<'a, T: ?Sized + 'a> Deref for BoxOrRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            BoxOrRef::Boxed(b) => &b,
            BoxOrRef::Ref(r) => r,
        }
    }
}

impl<'a, T: ?Sized + 'a> Borrow<T> for BoxOrRef<'a, T> {
    fn borrow(&self) -> &T {
        match self {
            BoxOrRef::Boxed(b) => b.borrow(),
            BoxOrRef::Ref(r) => r,
        }
    }
}

/* ---------- errors ---------- */

/// Errors that happen in individual `Fmt`s.
#[derive(Debug, PartialEq)]
pub enum SingleFmtError {
    /// Returned if a `Fmt` receives a flag it doesn't know how to handle.
    /// It's not actually used by the `impl`s for the standard types, but you
    /// can use it if you wish to be strict. Contains the erroneous flag.
    UnknownFlag(char),
    /// Returned if a `Fmt` receives an option it doesn't know how to handle.
    /// Again, standard types do not do this, they are not strict. Contains the
    /// erroneous option.
    UnknownOption(String),
    /// Returned when a given option (stored in the first field) contains an
    /// invalid value (stored in the second field). Standard types *do* use
    /// this. Contains a pair of erroneous option's name and value.
    InvalidOptionValue(String, String),
    /// Returned when a `Fmt` that is only used as a container to hold/produce
    /// other `Fmt`s via the dot access syntax is used directly. Contains the
    /// full path to the format unit used in such fashion.
    NamespaceOnlyFmt(String),
    /// Returned when a `Fmt`does not contain a requested sub-`Fmt`. Contains
    /// the full path to the child format unit.
    UnknownSubfmt(String),
}

/// Any error that can happen during formatting.
#[derive(Debug, PartialEq)]
pub enum FormattingError {
    // Parsing errors.
    /// Returned if a placeholder has an empty name. Contains the erroneous
    /// input.
    EmptyName(String),
    /// Retuned if an argument list is not closed off with a bracket. Contains
    /// the erroneous input.
    UnterminatedArgumentList(String),
    /// Returned if a placeholder is not terminated. Contains the erroneous
    /// input.
    UnterminatedPlaceholder(String),
    // Errors from single Fmts.
    /// A `SingleFmtError::UnknownFlag` is propagated as this.
    UnknownFlag(char),
    /// A `SingleFmtError::UnknownOption` is propagated as this.
    UnknownOption(String),
    /// A `SingleFmtError::InvalidOptionValue` is propagated as this.
    InvalidOptionValue(String, String),
    /// A `SingleFmtError::NamespaceOnlyFmt` is propagated as this.
    NamespaceOnlyFmt(String),
    // General errors.
    /// Returned when a requested `Fmt` does not exist (or cannot be created)
    /// in the format table. A `SingleFmtError::UnknownSubfmt` is also
    /// propagated as this. Contains the full path to the failed format unit.
    UnknownFmt(String),
}

impl From<SingleFmtError> for FormattingError {
    fn from(err: SingleFmtError) -> Self {
        match err {
            SingleFmtError::UnknownFlag(c) => FormattingError::UnknownFlag(c),
            SingleFmtError::UnknownOption(s) => FormattingError::UnknownOption(s),
            SingleFmtError::InvalidOptionValue(opt, val) => {
                FormattingError::InvalidOptionValue(opt, val)
            }
            SingleFmtError::NamespaceOnlyFmt(s) => FormattingError::NamespaceOnlyFmt(s),
            SingleFmtError::UnknownSubfmt(s) => FormattingError::UnknownFmt(s),
        }
    }
}

impl From<ParseError> for FormattingError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::EmptyNameSegment(s) => FormattingError::EmptyName(s),
            ParseError::UnterminatedArgumentList(s) => FormattingError::UnterminatedArgumentList(s),
            ParseError::UnterminatedPlaceholder(s) => FormattingError::UnterminatedPlaceholder(s),
        }
    }
}

/* ---------- key implementations ---------- */

impl<'a, T: Borrow<dyn Fmt + 'a>> Fmt for T {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        self.borrow().format(full_name, name, args, flags, options)
    }
}

impl<'a, T: FormatTable> FormatTable for &'a T {
    fn get_fmt<'b, 'c>(&'b self, name: &'c str) -> Option<BoxOrRef<'b, dyn Fmt>> {
        (*self).get_fmt(name)
    }
}

/* ---------- implementations of FormatTable for standard types ---------- */

impl<B: Borrow<dyn Fmt>> FormatTable for HashMap<String, B> {
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.get(name).map(|b| BoxOrRef::Ref(b.borrow()))
    }
}

impl<'a, B: Borrow<dyn Fmt>> FormatTable for HashMap<&'a str, B> {
    fn get_fmt<'b, 'c>(&'b self, name: &'c str) -> Option<BoxOrRef<'b, dyn Fmt>> {
        self.get(name).map(|r| BoxOrRef::Ref(r.borrow()))
    }
}

impl<B: Borrow<dyn Fmt>> FormatTable for Vec<B> {
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        if let Ok(index) = name.parse::<usize>() {
            if index < self.len() {
                Some(BoxOrRef::Ref(self[index].borrow()))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<A, B> FormatTable for (A, B)
where
    A: FormatTable,
    B: FormatTable,
{
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.0.get_fmt(name).or_else(|| self.1.get_fmt(name))
    }
}

impl<A, B, C> FormatTable for (A, B, C)
where
    A: FormatTable,
    B: FormatTable,
    C: FormatTable,
{
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.0
            .get_fmt(name)
            .or_else(|| self.1.get_fmt(name))
            .or_else(|| self.2.get_fmt(name))
    }
}

impl<A, B, C, D> FormatTable for (A, B, C, D)
where
    A: FormatTable,
    B: FormatTable,
    C: FormatTable,
    D: FormatTable,
{
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.0
            .get_fmt(name)
            .or_else(|| self.1.get_fmt(name))
            .or_else(|| self.2.get_fmt(name))
            .or_else(|| self.3.get_fmt(name))
    }
}

impl<A, B, C, D, E> FormatTable for (A, B, C, D, E)
where
    A: FormatTable,
    B: FormatTable,
    C: FormatTable,
    D: FormatTable,
    E: FormatTable,
{
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.0
            .get_fmt(name)
            .or_else(|| self.1.get_fmt(name))
            .or_else(|| self.2.get_fmt(name))
            .or_else(|| self.3.get_fmt(name))
            .or_else(|| self.4.get_fmt(name))
    }
}

impl<A, B, C, D, E, F> FormatTable for (A, B, C, D, E, F)
where
    A: FormatTable,
    B: FormatTable,
    C: FormatTable,
    D: FormatTable,
    E: FormatTable,
    F: FormatTable,
{
    fn get_fmt<'a, 'b>(&'a self, name: &'b str) -> Option<BoxOrRef<'a, dyn Fmt>> {
        self.0
            .get_fmt(name)
            .or_else(|| self.1.get_fmt(name))
            .or_else(|| self.2.get_fmt(name))
            .or_else(|| self.3.get_fmt(name))
            .or_else(|| self.4.get_fmt(name))
            .or_else(|| self.5.get_fmt(name))
    }
}

/* ---------- implementations of Fmt for standard types ---------- */

/// This instance is aware of the following flags:
/// * `y`, which changes the output from true/false to yes/no;
/// * `Y`, which changes the output to Y/N.
/// Common options are recognised.
impl Fmt for bool {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut res = if *self {
            if flags.contains(&'y') {
                "yes".to_string()
            } else if flags.contains(&'Y') {
                "Y".to_string()
            } else {
                "true".to_string()
            }
        } else if flags.contains(&'y') {
            "no".to_string()
        } else if flags.contains(&'Y') {
            "N".to_string()
        } else {
            "false".to_string()
        };
        util::apply_common_options(&mut res, options)?;
        Ok(res)
    }
}

/// This instance has no special flags.
/// Common options are recognised.
impl Fmt for char {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        _flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = self.to_string();
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `e`, which changes the output to the scientific, or exponential,
/// notation.
/// Common options are recognised.
/// Common numeric options are also recognised.
impl Fmt for f32 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut res: String;
        if flags.contains(&'e') {
            res = util::float_to_exp(*self, options)?;
        } else {
            res = util::float_to_normal(*self, options)?;
        }
        util::add_sign(&mut res, *self, flags)?;
        util::apply_common_options(&mut res, options)?;
        Ok(res)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `e`, which changes the output to scientific format.
/// Common options are recognized.
/// Common numeric options are also recognized.
impl Fmt for f64 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut res: String;
        if flags.contains(&'e') {
            res = util::float_to_exp(*self, options)?;
        } else {
            res = util::float_to_normal(*self, options)?;
        }
        util::add_sign(&mut res, *self, flags)?;
        util::apply_common_options(&mut res, options)?;
        Ok(res)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for i8 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for i16 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for i32 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for i64 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for i128 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which forces display of the sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes output hexadecimal;
/// Common and common numeric options are recognized.
impl Fmt for isize {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        util::add_sign(&mut s, *self, flags)?;
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance has no special flags.
/// Common options are recognised.
impl<'a> Fmt for &'a str {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        _flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = self.to_string();
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance has no special flags.
/// Common options are recognised.
impl Fmt for String {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        _flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = self.clone();
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for u8 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for u16 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for u32 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for u64 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for u128 {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/// This instance is aware of the following flags:
/// * `+`, which add a leading plus sign;
/// * `b`, which makes the output binary;
/// * `o`, which makes the output octal;
/// * `p`, which in combination with '`b`', '`o`' or '`x`' adds a base prefix
/// to the output.
/// * `x`, which makes the output hexadecimal.
/// Common and common numeric options are recognised.
impl Fmt for usize {
    fn format(
        &self,
        full_name: &[String],
        name: &[String],
        _args: &[String],
        flags: &[char],
        options: &HashMap<String, String>,
    ) -> Result<String, SingleFmtError> {
        if !name.is_empty() {
            return Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)));
        }
        let mut s = util::int_to_str(*self, flags, options)?;
        if flags.contains(&'+') {
            s.insert(0, '+');
        }
        util::apply_common_options(&mut s, options)?;
        Ok(s)
    }
}

/* ---------- tests for Fmts ---------- */

#[cfg(test)]
mod fmt_tests {
    test_suite! {
        name general;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt, FormattingError};

        test unknown_fmt() {
            let table: HashMap<&str, &Fmt> = HashMap::new();
            let s = table.format("i = {i}");
            assert_that!(&s, eq(Err(FormattingError::UnknownFmt("i".to_string()))));
        }

        test unknown_fmt_nested() {
            let i = 1;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i.a}");
            assert_that!(&s, eq(Err(FormattingError::UnknownFmt("i.a".to_string()))));
        }

        test integers_simple_1() {
            let i = 1;
            let j = 23;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            table.insert("j", &j);
            let s = table.format("i = {i}, j = {j}").unwrap();
            assert_that!(&s.as_str(), eq("i = 1, j = 23"));
        }

    }

    test_suite! {
        name boolean;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test flags() {
            let a = true;
            let b = false;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("a", &a);
            table.insert("b", &b);
            let s = table.format("{a}, {b:y}, {b:Y}").unwrap();
            assert_that!(&s.as_str(), eq("true, no, N"));
        }

    }

    test_suite! {
        name char;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test boring() {
            let c = 'z';
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("c", &c);
            let s = table.format("{c}, {c::width=l5}!").unwrap();
            assert_that!(&s.as_str(), eq("z, z    !"));
        }

    }

    test_suite! {
        name floats;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test exp_precision_neg() {
            let f: f32 = 1_234_567.891;
            let mut table: HashMap<String, &Fmt> = HashMap::new();
            table.insert("f".to_string(), &f);
            let s = table.format("{f:e+:prec=-1}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("+1.23457e6"));
        }

        test exp_precision_pos() {
            let f: f32 = 1000.123;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f:e:prec=2}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("1.00012e3"));
        }

        test exp_negative_power() {
            let f: f32 = 0.0625;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f:e}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("6.25e-2"));
        }

        test norm_rounding_up() {
            let f = 0.2;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f::round=up:prec=0}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("1"));
        }

        test norm_rounding_down() {
            let f = 0.8;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f::round=down:prec=0}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("0"));
        }

        test norm_rounding_usual() {
            let f = 0.5;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f::round=nearest:prec=0}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("1"));
        }

        test negative() {
            let f = -1.0;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("f", &f);
            let s = table.format("{f}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("-1"));
        }

    }

    test_suite! {
        name integers;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test basic() {
            let i = 10;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("10"));
        }

        test different_bases() {
            let i = 11;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i:b}, {i:o}, {i:x}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("1011, 13, b"));
        }

        test base_prefixes() {
            let i = 1;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i:bp}, {i:op}, {i:xp}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("0b1, 0o1, 0x1"));
        }

        test bases_for_negative_numbers() {
            let i = -11;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i:b}, {i:o}, {i:x}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("-1011, -13, -b"));
        }

        test rounding() {
            let i = 1235;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i::prec=-1}, {i::prec=-2}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("1240, 1200"));
        }

        test rounding_for_negatives() {
            let i = -1235;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("i", &i);
            let s = table.format("{i::prec=-1}, {i::prec=-2}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("-1240, -1200"));
        }

        test rounding_in_different_bases() {
            let o = 0o124;
            let b = 0b1101;
            let x = 0x1a2;
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("o", &o);
            table.insert("b", &b);
            table.insert("x", &x);
            let s1 = table.format("{o:op:prec=-1}, {o:op:prec=-2}").expect("Failed to parse 1");
            let s2 = table.format("{b:bp:prec=-1}, {b:bp:prec=-2}").expect("Failed to parse 2");
            let s3 = table.format("{x:xp:prec=-1}, {x:xp:prec=-2}").expect("Failed to parse 3");
            assert_that!(&s1.as_str(), eq("0o130, 0o100"));
            assert_that!(&s2.as_str(), eq("0b1110, 0b1100"));
            assert_that!(&s3.as_str(), eq("0x1a0, 0x200"));
        }

    }

    test_suite! {
        name common_options;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test width_left() {
            let string = "foobar";
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("s", &string);
            let s = table.format("{s::width=l10}").unwrap();
            assert_that!(&s.as_str(), eq("foobar    "));
        }

        test width_right() {
            let string = "foobar";
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("s", &string);
            let s = table.format("{s::width=r10}").unwrap();
            assert_that!(&s.as_str(), eq("    foobar"));
        }

        test width_center() {
            let string = "foobar";
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("s", &string);
            let s = table.format("{s::width=c10}").unwrap();
            assert_that!(&s.as_str(), eq("  foobar  "));
        }

        test truncate_left() {
            let string = "1234567890";
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("s", &string);
            let s = table.format("{s::truncate=l5}").unwrap();
            assert_that!(&s.as_str(), eq("67890"));
        }

        test truncate_right() {
            let string = "1234567890";
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("s", &string);
            let s = table.format("{s::truncate=r5}").unwrap();
            assert_that!(&s.as_str(), eq("12345"));
        }

    }

    test_suite! {
        name nested_fmts;
        use std::collections::HashMap;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt, FormattingError, SingleFmtError, util};

        struct Point {
            x: i32,
            y: i32
        }

        struct Line {
            start: Point,
            end: Point
        }

        impl Fmt for Point {
            fn format(&self,
                      full_name: &[String],
                      name: &[String],
                      args: &[String],
                      flags: &[char],
                      options: &HashMap<String, String>)
                -> Result<String, SingleFmtError>
                {
                    if name.is_empty() {
                        Err(SingleFmtError::NamespaceOnlyFmt(util::join_name(full_name)))
                    } else if name[0] == "x" {
                        self.x.format(full_name, &name[1..], args, flags, options)
                    } else if name[0] == "y" {
                        self.y.format(full_name, &name[1..], args, flags, options)
                    } else {
                        Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)))
                    }
                }
        }

        impl Fmt for Line {
            fn format(&self,
                      full_name: &[String],
                      name: &[String],
                      args: &[String],
                      flags: &[char],
                      options: &HashMap<String, String>)
                -> Result<String, SingleFmtError>
                {
                    if name.is_empty() {
                        Err(SingleFmtError::NamespaceOnlyFmt(util::join_name(full_name)))
                    } else if name[0] == "start" || name[0] == "a" {
                        self.start.format(full_name, &name[1..], args, flags, options)
                    } else if name[0] == "end" || name[0] == "b" {
                        self.end.format(full_name, &name[1..], args, flags, options)
                    } else {
                        Err(SingleFmtError::UnknownSubfmt(util::join_name(full_name)))
                    }
                }
        }

        test single_nested() {
            let a = Point { x: 0, y: 0 };
            let b = Point { x: 2, y: 10 };
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("a", &a);
            table.insert("b", &b);
            let s = table.format("{a.x}, {b.y}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("0, 10"));
        }

        test double_nested() {
            let line = Line {
                start: Point { x: 0, y: 2 },
                end: Point { x: 6, y: 10},
            };
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("line", &line);
            let s = table.format("{line.start.x}, {line.end.y}").expect("Failed to format");
            assert_that!(&s.as_str(), eq("0, 10"));
        }

        test namespace_only() {
            let p = Point { x: 1, y: 2 };
            let mut table: HashMap<&str, &Fmt> = HashMap::new();
            table.insert("p", &p);
            let s = table.format("{p}");
            assert_that!(&s, eq(Err(FormattingError::NamespaceOnlyFmt("p".to_string()))));
        }

    }

}

/* ---------- tests for FormatTables ---------- */

#[cfg(test)]
mod table_tests {
    test_suite! {
        name vec;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt, FormattingError};

        test unknown_fmt_1() {
            let i = 1;
            let j = 2;
            let table: Vec<&Fmt> = vec![&i, &j];
            let err = table.format("{10}").expect_err("Unexpectedly found a fmt");
            assert_that!(&err, eq(FormattingError::UnknownFmt("10".to_string())));
        }

        test unknown_fmt_2() {
            let i = 1;
            let j = 2;
            let table: Vec<&Fmt> = vec![&i, &j];
            let err = table.format("{-3}").expect_err("Unexpectedly found a fmt");
            assert_that!(&err, eq(FormattingError::UnknownFmt("-3".to_string())));
        }

        test boring() {
            let i = 1;
            let j = 2;
            let table: Vec<&Fmt> = vec![&i, &j];
            let s = table.format("{0}, {1}").expect("Failed to format");
            assert_that!(&s, eq("1, 2".to_string()));
        }

    }

    test_suite! {
        name tuples;
        use galvanic_assert::matchers::*;
        use {FormatTable, Fmt};

        test defaulting() {
            let a: Vec<Box<Fmt>> = vec![Box::new(-1), Box::new(-2)];
            let b: Vec<Box<Fmt>> = (0..10_i32).map(|i| Box::new(i) as Box<Fmt>).collect();
            let s = (a, b).format("{5}").expect("Failed");
            assert_that!(&s, eq("5".to_string()));
        }

        test precedence() {
            let a: Vec<Box<Fmt>> = vec![Box::new(1)];
            let b: Vec<Box<Fmt>> = vec![Box::new(10)];
            let s = (a, b).format("{0}").expect("Failed");
            assert_that!(&s, eq("1".to_string()));
        }

    }

}
