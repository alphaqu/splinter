use ahash::AHashMap;
use lazy_static::lazy_static;
use syn::__private::ToTokens;
use syn::{LitInt, LitStr};

lazy_static! {
    static ref LOOKUP: AHashMap<String, String> = create_lookup();
}

#[proc_macro]
pub fn icon(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit = syn::parse::<LitStr>(item).expect("Failed to parse icon name");
    let name = lit.value();
    match LOOKUP.get(&name) {
        None => {
            panic!("Icon \"{name}\" does not exist.")
        }
        Some(codepoint) => LitInt::new(codepoint, lit.span()).to_token_stream().into(),
    }
}

fn create_lookup() -> AHashMap<String, String> {
    let codepoints = include_str!("codepoints");
    let mut lookup = AHashMap::new();
    for entry in codepoints.split('\n') {
        let (name, codepoint) = entry.split_once(' ').unwrap();
        lookup.insert(name.to_string(), format!("0x{codepoint}"));
    }
    lookup
}
