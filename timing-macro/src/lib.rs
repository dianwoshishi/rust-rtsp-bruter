use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, FnArg, GenericParam, ItemFn, Pat, PatIdent, ReturnType};

/// 处理所有四种情况的计时宏：
/// 1. 同步函数（有参数/无参数）
/// 2. 异步函数（有参数/无参数）
#[proc_macro_attribute]
pub fn timing(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入函数
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let wrapped_fn_name = format_ident!("{}_wrapped", fn_name); // 包装函数名
    let vis = &input_fn.vis; // 原函数可见性（如 pub）
    let sig = &input_fn.sig; // 函数签名（含参数、返回值等）
    let is_async = sig.asyncness.is_some(); // 判断是否为异步函数
    let block = &input_fn.block; // 原函数体
    let attrs = &input_fn.attrs; // 原函数属性（如 #[inline]）

    // 1. 处理泛型参数（支持带泛型的函数）
    let generic_params: Vec<&GenericParam> = sig.generics.params.iter().collect();
    // 2. 处理 where 子句（支持泛型约束）
    let where_clause = sig.generics.where_clause.as_ref()
        .map(|wc| quote! { where #wc })
        .unwrap_or_default();

    // 3. 提取参数模式（用于调用包装函数，仅参数名，适配有/无参数）
    let arg_patterns: Vec<proc_macro2::TokenStream> = sig.inputs.iter()
        .map(|arg| match arg {
            // 普通参数（如 a: i32 → 提取 a）
            FnArg::Typed(pat_type) => pat_type.pat.to_token_stream(),
            // self 参数（如 &self / &mut self → 提取 self）
            FnArg::Receiver(_) => Pat::Ident(PatIdent {
                attrs: Default::default(),
                by_ref: None,
                mutability: None,
                ident: syn::Ident::new("self", proc_macro2::Span::call_site()),
                subpat: None,
            }).to_token_stream(),
        })
        .collect();

    // 4. 提取完整参数（用于定义包装函数，含类型，适配有/无参数）
    let fn_inputs: Vec<&FnArg> = sig.inputs.iter().collect();

    // 5. 提取返回值类型（适配有/无返回值）
    let return_type = match &sig.output {
        ReturnType::Default => quote! { () }, // 无返回值（默认 ()）
        ReturnType::Type(_, ty) => ty.to_token_stream(), // 有返回值（如 i32）
    };

    // 生成代码：根据同步/异步分支处理
    let output = if is_async {
        // 异步函数处理
        quote! {
            #(#attrs)* // 保留原函数属性
            #vis #sig { // 保留原函数可见性和签名
                let start_time = std::time::Instant::now();
                // 调用包装函数（用参数模式，支持有/无参数）
                let result = #wrapped_fn_name(#(#arg_patterns),*).await;
                let duration = start_time.elapsed();
                println!("\nFunction `{}` executed in {:?}", stringify!(#fn_name), duration);
                result // 返回原函数结果
            }

            // 异步包装函数（私有，含完整参数和返回值）
            async fn #wrapped_fn_name<#(#generic_params),*>(
                #(#fn_inputs),* // 完整参数（含类型）
            ) -> #return_type #where_clause { // 返回值和泛型约束
                #block // 原函数体
            }
        }
    } else {
        // 同步函数处理
        quote! {
            #(#attrs)* // 保留原函数属性
            #vis #sig { // 保留原函数可见性和签名
                let start_time = std::time::Instant::now();
                // 调用包装函数（用参数模式，支持有/无参数）
                let result = #wrapped_fn_name(#(#arg_patterns),*);
                let duration = start_time.elapsed();
                println!("\nFunction `{}` executed in {:?}\n", stringify!(#fn_name), duration);
                result // 返回原函数结果
            }

            // 同步包装函数（私有，含完整参数和返回值）
            fn #wrapped_fn_name<#(#generic_params),*>(
                #(#fn_inputs),* // 完整参数（含类型）
            ) -> #return_type #where_clause { // 返回值和泛型约束
                #block // 原函数体
            }
        }
    };

    TokenStream::from(output)
}
