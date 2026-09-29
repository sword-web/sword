use crate::controllers::shared::CMetaStack;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr, FnArg, Ident, ItemFn, LitStr, Type};

/// Which job handler attribute is being expanded.
pub enum ScheduleKind {
    Cron,
    Repeated,
    Once,
}

/// Parsed attribute payload.
pub enum ScheduleSpec {
    Cron(LitStr),
    Repeated(Expr),
    Once(Expr),
}

/// Everything generation needs about a job handler method.
pub struct ParsedHandler {
    pub item: ItemFn,
    pub fn_name: Ident,
    pub controller_ident: Ident,
    pub controller_name: String,
    pub schedule: ScheduleSpec,
}

/// Entry point for `#[cron]` / `#[repeated]` / `#[once]`.
pub fn expand_handle(
    kind: ScheduleKind,
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let Some(parsed) = parse_handle(kind, attr, item.clone())? else {
        return Ok(item); // not inside a job controller
    };

    generate_handle(parsed)
}

fn parse_handle(
    kind: ScheduleKind,
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<Option<ParsedHandler>> {
    // Handshake: leave methods untouched outside a job controller.
    let Some(controller_name) = CMetaStack::get("job_controller", "controller_name") else {
        return Ok(None);
    };

    let schedule = parse_schedule(kind, attr)?;
    let item_fn: ItemFn = syn::parse(item)?;
    let fn_name = parse_method(&item_fn)?;
    let controller_ident: Ident = syn::parse_str(&controller_name)?;

    Ok(Some(ParsedHandler {
        item: item_fn,
        fn_name,
        controller_ident,
        controller_name,
        schedule,
    }))
}

fn parse_schedule(kind: ScheduleKind, attr: TokenStream) -> syn::Result<ScheduleSpec> {
    match kind {
        ScheduleKind::Cron => {
            let lit: LitStr = syn::parse(attr).map_err(|err| {
                Error::new(
                    err.span(),
                    "`#[cron]` expects a string literal, e.g. #[cron(\"*/5 * * * * *\")]",
                )
            })?;

            if lit.value().trim().is_empty() {
                return Err(Error::new_spanned(
                    &lit,
                    "cron expression must not be empty",
                ));
            }

            Ok(ScheduleSpec::Cron(lit))
        }

        ScheduleKind::Repeated => {
            let expr: Expr = syn::parse(attr).map_err(|err| {
                Error::new(
                    err.span(),
                    "`#[repeated]` expects a duration, e.g. #[repeated(Duration::from_secs(5))]",
                )
            })?;

            Ok(ScheduleSpec::Repeated(expr))
        }

        ScheduleKind::Once => {
            let expr: Expr = syn::parse(attr).map_err(|err| {
                Error::new(
                    err.span(),
                    "`#[once]` expects a duration, e.g. #[once(Duration::from_secs(30))]",
                )
            })?;

            Ok(ScheduleSpec::Once(expr))
        }
    }
}

/// Requires `async fn name(&self, uuid: Uuid, scheduler: JobScheduler)`.
fn parse_method(item_fn: &ItemFn) -> syn::Result<Ident> {
    if item_fn.sig.asyncness.is_none() {
        return Err(Error::new_spanned(
            &item_fn.sig,
            "a job handler must be `async`",
        ));
    }

    let mut inputs = item_fn.sig.inputs.iter();

    match inputs.next() {
        Some(FnArg::Receiver(_)) => {}
        _ => {
            return Err(Error::new_spanned(
                &item_fn.sig,
                "a job handler must take `&self` as its first parameter",
            ));
        }
    }

    let uuid = inputs.next().ok_or_else(|| {
        Error::new_spanned(
            &item_fn.sig,
            "expected `uuid: Uuid` as the second parameter",
        )
    })?;

    let scheduler = inputs.next().ok_or_else(|| {
        Error::new_spanned(
            &item_fn.sig,
            "expected `scheduler: JobScheduler` as the third parameter",
        )
    })?;

    if inputs.next().is_some() {
        return Err(Error::new_spanned(
            &item_fn.sig,
            "a job handler must be exactly `fn(&self, uuid: Uuid, scheduler: JobScheduler)`",
        ));
    }

    expect_type(uuid, "Uuid")?;
    expect_type(scheduler, "JobScheduler")?;

    Ok(item_fn.sig.ident.clone())
}

fn expect_type(arg: &FnArg, expected: &str) -> syn::Result<()> {
    let FnArg::Typed(pat) = arg else {
        return Ok(());
    };

    let last = match pat.ty.as_ref() {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    };

    if last.as_deref() != Some(expected) {
        return Err(Error::new_spanned(
            &pat.ty,
            format!("this parameter must be of type `{expected}`"),
        ));
    }

    Ok(())
}

/// Emits the original method plus the `JobFunctionRegistrar` that builds its
/// `Job`. The runtime then adds and starts it.
fn generate_handle(parsed: ParsedHandler) -> syn::Result<TokenStream> {
    let ParsedHandler {
        item,
        fn_name,
        controller_ident,
        controller_name,
        schedule,
    } = parsed;

    let (constructor, schedule_arg) = match &schedule {
        ScheduleSpec::Cron(lit) => (
            quote!(::sword::internal::jobs::Job::new_async),
            quote!(#lit),
        ),
        ScheduleSpec::Repeated(expr) => (
            quote!(::sword::internal::jobs::Job::new_repeated_async),
            quote!(#expr),
        ),
        ScheduleSpec::Once(expr) => (
            quote!(::sword::internal::jobs::Job::new_one_shot_async),
            quote!(#expr),
        ),
    };

    let registration_name = format_ident!(
        "__SWORD_JOB_{}_{}",
        controller_name.replace("::", "_"),
        fn_name
    );

    let expanded = quote! {
        #item

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::jobs::JobFunctionRegistrar {
                    controller_type_id: ::std::any::TypeId::of::<#controller_ident>(),
                    handler: |state: &::sword::internal::core::State|
                        -> ::sword::internal::jobs::Job
                    {
                        let controller = state
                            .borrow::<#controller_ident>()
                            .unwrap_or_else(|err| {
                                ::sword::internal::core::sword_error! {
                                    title: "Failed to retrieve job controller from state",
                                    reason: err,
                                    context: {
                                        "controller" => #controller_name,
                                        "job" => stringify!(#fn_name),
                                    },
                                    hints: [
                                        "Ensure the controller is registered via Module::register_controllers",
                                    ],
                                }
                            });

                        #constructor(#schedule_arg, move |uuid, scheduler| {
                            let controller = controller.clone();

                            ::std::boxed::Box::pin(async move {
                                controller.#fn_name(uuid, scheduler).await
                            })
                        })
                        .unwrap_or_else(|err| {
                            ::sword::internal::core::sword_error! {
                                title: "Failed to register job",
                                reason: err,
                                context: {
                                    "controller" => #controller_name,
                                    "job" => stringify!(#fn_name),
                                },
                                hints: [
                                    "Check the cron expression or duration passed to the job attribute",
                                ],
                            }
                        })
                    },
                }
            }
        };
    };

    Ok(TokenStream::from(expanded))
}
