use scylla::frame::response::result::CqlValue;
use std::borrow::Cow;

fn fmt_map<'a>(col: &'a Vec<(CqlValue, CqlValue)>, into: &'a mut String) {
    into.push('{');
    for (key, value) in col {
        fmt_entry(key, into);
        into.push(':');
        into.push(' ');
        fmt_entry(value, into);
        if value != &col.last().unwrap().0 {
            into.push_str(", ");
        }
    }
    into.push('}');
}

fn fmt_udt<'a>(col: &'a Vec<(String, Option<CqlValue>)>, into: &'a mut String) {
    into.push('{');
    for (key, value) in col {
        into.push_str(key);
        into.push(':');
        into.push(' ');
        fmt_opt_entry(value, into);
        if key != &col.last().unwrap().0 {
            into.push_str(", ");
        }
    }
    into.push('}');
}

fn fmt_vec<'a>(col: &'a Vec<CqlValue>, into: &'a mut String) {
    into.push('{');
    for value in col {
        fmt_entry(value, into);
        if value != col.last().unwrap() {
            into.push_str(", ");
        }
    }
    into.push('}');
}

fn fmt_tpl<'a>(col: &'a Vec<Option<CqlValue>>, into: &'a mut String) {
    into.push('{');
    for value in col {
        fmt_opt_entry(value, into);
        if value != col.last().unwrap() {
            into.push_str(", ");
        }
    }
    into.push('}');
}

fn fmt_opt_entry<'a>(entry: &'a Option<CqlValue>, into: &'a mut String) {
    match entry {
        None => into.push_str("null"),
        Some(entry) => fmt_entry(entry, into)
    };
}

fn fmt_entry<'a>(entry: &'a CqlValue, into: &'a mut String) {
    match entry {
        CqlValue::List(col) | CqlValue::Set(col) => {
            fmt_vec(col, into);
        }
        CqlValue::Map(col) => {
            fmt_map(col, into);
        }
        CqlValue::UserDefinedType { fields, .. } => {
            fmt_udt(fields, into);
        }
        _ => {
            into.push('\'');
            into.push_str(&fmt(entry));
            into.push('\'');
        }
    };
}

fn apply<'a, T: Fn(&mut String) -> ()>(a: T) -> String {
    let mut out = String::new();
    a(&mut out);

    out
}

pub fn fmt_opt(col: &Option<CqlValue>) -> Cow<str> {
    match col {
        None => Cow::Borrowed("null"),
        Some(col) => fmt(col)
    }
}

pub fn fmt(col: &CqlValue) -> Cow<str> {
    match col {
        CqlValue::Ascii(col) | CqlValue::Text(col) => Cow::Borrowed(col),
        CqlValue::Boolean(col) => Cow::Borrowed(if *col { "true" } else { "false" }),
        CqlValue::Blob(_) => Cow::Borrowed("<blob>"),
        CqlValue::Counter(col) => Cow::Owned(col.0.to_string()),
        CqlValue::Decimal(col) => Cow::Owned(col.to_string()),
        CqlValue::Date(col) => Cow::Owned(col.to_string()),
        CqlValue::Double(col) => Cow::Owned(col.to_string()),
        CqlValue::Empty => Cow::Borrowed(""),
        CqlValue::Float(col) => Cow::Owned(col.to_string()),
        CqlValue::Int(col) => Cow::Owned(col.to_string()),
        CqlValue::BigInt(col) => Cow::Owned(col.to_string()),
        CqlValue::Timestamp(col) => Cow::Owned(col.to_string()),
        CqlValue::Inet(col) => Cow::Owned(col.to_string()),
        CqlValue::List(col) | CqlValue::Set(col) => Cow::Owned(apply(|s| fmt_vec(col, s))),
        CqlValue::Map(col) => Cow::Owned(apply(|s| fmt_map(col, s))),
        CqlValue::UserDefinedType { fields, .. } => Cow::Owned(apply(|s| fmt_udt(fields, s))),
        CqlValue::SmallInt(col) => Cow::Owned(col.to_string()),
        CqlValue::TinyInt(col) => Cow::Owned(col.to_string()),
        CqlValue::Time(col) => Cow::Owned(col.to_string()),
        CqlValue::Timeuuid(col) => Cow::Owned(col.to_string()),
        CqlValue::Tuple(col) => Cow::Owned(apply(|s| fmt_tpl(col, s))),
        CqlValue::Uuid(col) => Cow::Owned(col.to_string()),
        CqlValue::Varint(col) => Cow::Owned(col.to_string()),
    }
}

