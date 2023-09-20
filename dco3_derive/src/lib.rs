extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromResponse)]
pub fn from_response_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Used for generating code
    let name = input.ident;

    // Generate the code to implement `FromResponse`
    let expanded = quote! {

    #[async_trait::async_trait]
    impl crate::utils::FromResponse for #name {
        async fn from_response(response: reqwest::Response) -> Result<Self, crate::DracoonClientError> {
            crate::utils::parse_body::<Self, crate::auth::DracoonErrorResponse>(response).await
        }
      }
    };

    // Return the generated code
    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
