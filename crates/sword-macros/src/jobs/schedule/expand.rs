use super::parsing::{ParsedScheduleHandler, Schedule};
use crate::shared::{Result, location, paths::*};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn expand_schedule_handler(schedule: ParsedScheduleHandler) -> Result<TokenStream> {
    let ParsedScheduleHandler {
        item_fn,
        fn_name,
        controller_ty,
        schedule,
    } = schedule;

    let location = location::tokens(fn_name.span(), &fn_name.to_string());

    let (job_constructor, schedule_arg) = match &schedule {
        Schedule::Cron(lit) => (quote!(#Job::new_async), quote!(#lit)),
        Schedule::Repeated(expr) => (quote!(#Job::new_repeated_async), quote!(#expr)),
        Schedule::Once(expr) => (quote!(#Job::new_one_shot_async), quote!(#expr)),
    };

    let registration_name = format_ident!(
        "__SWORD_JOB_{}_{}",
        &controller_ty.to_string().replace("::", "_"),
        &fn_name
    );

    let expanded = quote! {
        #item_fn

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            #inventory::submit! {
                #JobFunctionRegistrar {
                    controller_type_id: #TypeId::of::<#controller_ty>(),
                    handler: |state: &#State| -> ::std::result::Result<#Job, #DiagnosticError> {
                        let controller = state.borrow::<#controller_ty>().map_err(|err| {
                            #sword_error! {
                                code: "sword.build.jobs",
                                phase: #DiagnosticPhase::Build,
                                target: "job schedule",
                                title: "Failed to retrieve job controller from state",
                                reason: err,
                                source_location: {
                                    file: (#location).file(),
                                    line: (#location).line(),
                                    column: (#location).column(),
                                    length: (#location).length(),
                                    message: format!("job `{}`", stringify!(#fn_name)),
                                },
                                context: {
                                    "controller" => #stringify!(#controller_ty),
                                    "job" => stringify!(#fn_name),
                                },
                                hints: [
                                    "Ensure the controller is registered via Module::register_controllers",
                                ],
                                cause: err,
                            }
                        })?;

                        #job_constructor(#schedule_arg, move |uuid, scheduler| {
                            let controller = #Arc::clone(&controller);

                            #Box::pin(async move {
                                controller.#fn_name(uuid, scheduler).await
                            })
                        })
                        .map_err(|err| {
                            #sword_error! {
                                code: "sword.build.jobs",
                                phase: #DiagnosticPhase::Build,
                                target: "job schedule",
                                title: "Failed to register job",
                                reason: err,
                                source_location: {
                                    file: (#location).file(),
                                    line: (#location).line(),
                                    column: (#location).column(),
                                    length: (#location).length(),
                                    message: format!("job `{}`", stringify!(#fn_name)),
                                },
                                context: {
                                    "controller" => #stringify!(#controller_ty),
                                    "job" => stringify!(#fn_name),
                                },
                                hints: [
                                    "Check the cron expression or duration passed to the job attribute",
                                ],
                                cause: err,
                            }
                        })
                    }
                }
            }
        };
    };

    Ok(expanded)
}
