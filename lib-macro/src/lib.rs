use std::collections::BTreeMap;

use proc_macro::{TokenStream};
use quote::{ToTokens, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta, Lit};

#[allow(unused_variables)]
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn AMFapi(args: TokenStream, input: TokenStream ) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let func = parse_macro_input!(input as ItemFn);
    
    let args_map = args.into_iter()
        .map(|arg| {
            use NestedMeta::{Meta};
            use syn::Meta::NameValue;

            if let Meta(NameValue(meta)) = arg {
                let key = meta.path
                    .into_token_stream()
                    .to_string();
                return Some((key, meta.lit));
            }

            None
        })
        .filter(Option::is_some)
        .flatten()
        .collect::<BTreeMap<_,_>>();

    let target_uri = args_map.get("target")
        .expect("必须提供target (target_uri)");
    let target_uri = 
        if let Lit::Str(s) = target_uri {
            s.value()
        } else {
            panic!("target必须是`&'static str`");
        };

    let response_uri = args_map.get("response")
        .expect("必须提供response (response_uri)");
    let response_uri = 
        if let Lit::Str(s) = response_uri {
            s.value()
        } else {
            panic!("target必须是`&'static str`");
        };

    // let data_type = args_map.get("data_type")
    //     .expect("必须提供data_type");

    todo!();

    #[allow(unreachable_code)]
    quote!(#func).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
