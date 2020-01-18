
extern crate proc_macro;
use crate::proc_macro::TokenStream;

use quote::{quote,format_ident};
//use proc_macro_roids::{DeriveInputStructExt, FieldExt, IdentExt};
use syn;
use syn::{parse_macro_input, parse_quote, DeriveInput, AttributeArgs, NestedMeta, Lit};

#[proc_macro_attribute]
pub fn atat(attr: TokenStream, input: TokenStream) -> TokenStream {
    //Load up the incoming struct and the attached attributes
    let ast = syn::parse(input).unwrap();
    let args = parse_macro_input!(attr as AttributeArgs);

    //Get the end of command character
    let end_line = if let NestedMeta::Lit(Lit::Str(ref end_line_tokens)) = args[0] {
        end_line_tokens.value().clone()
    } else {
        "\n".to_string()
    };
  
    //Build the state enum which has only the elements with no sub variables
    let state = impl_atat_state(&ast);

    // Build the trait implementation
    let create = impl_atat_create(&ast,end_line);
    let parse = impl_atat_parse(&ast);

    let output = quote!{
        #state
        #create
        #parse
    };
    println!("{}", output.to_string());
    output.into()

}

fn impl_atat_state(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = &ast.data;
    let mut new_variants = Vec::new();
    if let syn::Data::Enum(members) = &data {

        for variant in &members.variants {
            new_variants.push(variant.ident.clone());
        }

    }

    let gen = quote! {
        #[derive(Debug,Clone,Copy)]
        #vis enum #name {
            #(#new_variants),*,
            RawCommandPassthrough
        }
    };
    gen
}

fn impl_atat_create(ast: &DeriveInput, end_line: String) -> proc_macro2::TokenStream {
    let name = format_ident!("{}Create", ast.ident);

    let state_name = ast.ident.clone();
    let vis = &ast.vis;
    let data = &ast.data;

    use syn::{Variant, Arm, Fields, Data::Enum, punctuated::Punctuated, Type::Tuple, token::Comma};

    //Set up 2 new variables, the list of variants to go into the enum
    let mut new_variants_enum: Vec<Variant> = Vec::new();
    //And the variants to go in the match
    let mut new_variants_match: Vec<Arm> = Vec::new();
    if let Enum(members) = &data {

        for variant in &members.variants {

            //Clone the existing variant to the new one
            let mut new_variant = variant.clone();

            //Get the fields we want based on the enum structure
            if let Fields::Unnamed(fields) = &mut new_variant.fields {
                if fields.unnamed.len() >= 2 {

                    //Set up the blank field variables
                    let mut fields_create_parsable: Punctuated<ParsableUnnamedField, Comma> = Punctuated::new();
                    let mut fields_create_writable: Punctuated<syn::Field, Comma> = Punctuated::new();
                    let mut fields_create_match = Vec::new();

                    //The second field of the tuple is the input parameters
                    let field_type = &fields.unnamed[1].ty;

                    if let Tuple(tuple_struct) = field_type {
                        for element in &tuple_struct.elems {
                            let new_field_type = parse_quote!(#element);
                            fields_create_parsable.push(new_field_type);

                            //Add the elements to the match pattern as identifiers
                            let tuple_pos = format!("element_{}",fields_create_match.len());
                            fields_create_match.push(format_ident!("{}",tuple_pos));
                        }
                    }

                    //Map the parseable fields to the writable ones
                    fields_create_writable.extend(fields_create_parsable.into_iter().map(|p| p.field));

                    //Grab the ident as the expression
                    let variant_ident = &new_variant.ident;

                    //Convert the name of the enum to the correct command
                    let mut variant_fmt_string = variant_ident.to_string();
                    if variant_fmt_string.ends_with("__") {
                        variant_fmt_string += "=?";
                    } else if variant_fmt_string.ends_with("_") {
                        variant_fmt_string += "?";
                    } else {
                        variant_fmt_string += "=";
                    }

                    //Prepend the AT command
                    if ! variant_fmt_string.ends_with("_") {
                        variant_fmt_string.insert_str(0,"AT+")
                    }

                    //Clear all the underscores in the command
                    variant_fmt_string.retain(|c| c != '_');

                    //Add in the various braces for the incoming variables 
                    for _i in 0..fields_create_match.len() {
                        variant_fmt_string += "{}";
                    }

                    //Add the end of line character
                    variant_fmt_string += end_line.as_str();

                    let new_variant_match = parse_quote!(
                        Self::#variant_ident(#(#fields_create_match),*) => {
                           
                            uwrite!(buffer_out,#variant_fmt_string,#(#fields_create_match),*).map_err(|_e| ATATError::CreateError )?;

                            Ok(#state_name::#variant_ident)
                        }
                    );


                    //Overwrite the fields on the variant with the new ones
                    fields.unnamed = fields_create_writable;

                    //Push the new variant onto the new list
                    new_variants_enum.push(new_variant);

                    //Push the match output to the new list
                    new_variants_match.push(new_variant_match);
                } else {
                    //todo!()
                }
            } else {
                //todo!()
            }
        }

    }

    //
    let gen = quote! {
        use ufmt::uwrite;

        #[derive(Debug)]
        #vis enum #name<'b> {
            #(#new_variants_enum),*,
            RawCommandPassthrough (&'b[u8]),
        }
        impl<'a> ATATCreate<'a>  for #name<'_> {
            type ATATState = #state_name;
            type Buffer = &'a mut Cursor<[u8]>;
            fn create_command(&self, buffer_out: Self::Buffer) -> Result<#state_name,ATATError> {
                
                match self {
                    #(#new_variants_match),*,
                    Self::RawCommandPassthrough (data) => {
                        for i in data.iter() {
                            uwrite!(buffer_out,"{}",i).map_err(|_e| ATATError::CreateError )?;
                        }

                        Ok(#state_name::RawCommandPassthrough)
                    },
                    _ => Err(ATATError::UnknownCommandError)
                }
                
            }
        }
    };
    gen
}

fn impl_atat_parse(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = format_ident!("{}Parse", ast.ident);

    let state_name = ast.ident.clone();
    let vis = &ast.vis;
    let data = &ast.data;

    use syn::{Variant, Arm, Fields, Data::Enum, punctuated::Punctuated, Type::Tuple, token::Comma};

    //Set up 2 new variables, the list of variants to go into the enum
    let mut new_variants_enum: Vec<Variant> = Vec::new();
    //And the variants to go in the match
    let mut new_variants_match: Vec<Arm> = Vec::new();
    if let Enum(members) = &data {

        for variant in &members.variants {

            //Clone the existing variant to the new one
            let mut new_variant = variant.clone();

            //Get the fields we want based on the enum structure
            if let Fields::Unnamed(fields) = &mut new_variant.fields {
                if fields.unnamed.len() >= 3 {

                    //Set up the blank field variables
                    let mut fields_parse_parsable: Punctuated<ParsableUnnamedField, Comma> = Punctuated::new();
                    let mut fields_parse_writable: Punctuated<syn::Field, Comma> = Punctuated::new();
                    let mut fields_parse_match = Vec::new();

                    //The second field of the tuple is the input parameters
                    let field_type = &fields.unnamed[2].ty;

                    if let Tuple(tuple_struct) = field_type {
                        for element in &tuple_struct.elems {
                            let new_field_type = parse_quote!(#element);
                            fields_parse_parsable.push(new_field_type);

                            //Add the elements to the match pattern as identifiers
                            let tuple_pos = format!("element_{}",fields_parse_match.len());
                            fields_parse_match.push(format_ident!("{}",tuple_pos));
                        }
                    }

                    //Map the parseable fields to the writable ones
                    fields_parse_writable.extend(fields_parse_parsable.into_iter().map(|p| p.field));

                    //Grab the ident as the expression
                    let variant_ident = &new_variant.ident;

                    //Convert the name of the enum to the correct command
                    /*let mut variant_fmt_string = variant_ident.to_string();
                    if variant_fmt_string.ends_with("__") {
                        variant_fmt_string += "=?";
                    } else if variant_fmt_string.ends_with("_") {
                        variant_fmt_string += "?";
                    } else {
                        variant_fmt_string += "=";
                    }

                    //Prepend the AT command
                    if ! variant_fmt_string.ends_with("_") {
                        variant_fmt_string.insert_str(0,"AT+")
                    }

                    //Clear all the underscores in the command
                    variant_fmt_string.retain(|c| c != '_');

                    //Add in the various braces for the incoming variables 
                    for _i in 0..fields_parse_match.len() {
                        variant_fmt_string += "{}";
                    }

                    //Add the end of line character
                    variant_fmt_string += end_line.as_str();*/

                    let new_variant_match = parse_quote!(
                        #state_name::#variant_ident => {
                           
                            //uwrite!(buffer_out,#variant_fmt_string,#(#fields_parse_match),*).map_err(|_e| ATATError::CreateError )?;

                            //Ok(#state_name::#variant_ident)
                            Err(ATATError::UnknownCommandError)
                        }
                    );


                    //Overwrite the fields on the variant with the new ones
                    fields.unnamed = fields_parse_writable;

                    //Push the new variant onto the new list
                    new_variants_enum.push(new_variant);

                    //Push the match output to the new list
                    new_variants_match.push(new_variant_match);
                } else {
                    //todo!()
                }
            } else {
                //todo!()
            }
        }

    }

    let gen = quote! {
        #vis enum #name {
            #(#new_variants_enum),*,
            //RawCommandPassthrough (&'b[u8]),
        }
        impl<'a>  ATATParse<'a>  for #name {
            type ATATState = #state_name;
            type Buffer = &'a mut Cursor<[u8]>;
            fn parse_response(state: #state_name, buffer_in: Self::Buffer) -> Result<Self,ATATError> {
                match state {
                    #(#new_variants_match),*,
                    /*Self::RawCommandPassthrough (data) => {
                        for i in data.iter() {
                            uwrite!(buffer_out,"{}",i).map_err(|_e| ATATError::CreateError )?;
                        }

                        Ok(#state_name::RawCommandPassthrough)
                    },*/
                    _ => Err(ATATError::UnknownCommandError)
                }
            }
        }
    };
    gen
}

struct ParsableUnnamedField {
    pub field: Field,
}

use syn::{parse, Field};
use syn::parse::{Parse,ParseStream};

impl Parse for ParsableUnnamedField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let field = Field::parse_unnamed(input)?;

        Ok(ParsableUnnamedField {
            field,
        })
    }
}

//https://doc.rust-lang.org/book/ch19-06-macros.html
//https://tinkering.xyz/introduction-to-proc-macros/
//https://docs.rs/proc_macro_roids/0.6.0/proc_macro_roids/
//https://cprimozic.net/blog/writing-a-hashmap-to-struct-procedural-macro-in-rust/
//https://danielkeep.github.io/tlborm/book/README.html
//https://cbreeden.github.io/Macros11/
//https://docs.rs/enum-as-inner/0.3.0/enum_as_inner/
//https://lib.rs/development-tools/procedural-macro-helpers
//https://github.com/dtolnay/syn/issues/651



