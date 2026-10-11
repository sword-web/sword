use crate::shared::{self, Result, cmeta::CMetaStack};

use darling::Error;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::format_ident;
use syn::{Expr, FnArg, Ident, ItemFn, LitStr, spanned::Spanned};

pub enum ScheduleKind {
    Cron,
    Repeated,
    Once,
}

pub enum Schedule {
    Cron(LitStr),
    Repeated(Expr),
    Once(Expr),
}

pub struct ParsedScheduleHandler {
    pub item_fn: ItemFn,
    pub fn_name: Ident,
    pub controller_ty: Ident,
    pub schedule: Schedule,
}

impl ParsedScheduleHandler {
    pub fn parse(kind: ScheduleKind, attr: TokenStream, item: TokenStream) -> Result<Self> {
        let item_fn = syn::parse::<ItemFn>(item).map(Self::validate_method)??;
        let schedule = Schedule::parse(kind, attr)?;

        let Some(controller_ty) = CMetaStack::get("job_controller", "controller_ty") else {
            let error = "\n[ERROR] This attribute macro must be used inside a Job Controller impl block.\n\
                \n\
                Make sure:\n\
                1. The struct has #[controller(kind = Controller::JobHandler)] attribute\n\
                2. The struct is defined BEFORE the impl block\n\
                3. The impl block is for the same struct\n";

            return Err(Error::custom(error).with_span(&Span::call_site()));
        };

        let controller_ty = format_ident!("{}", controller_ty);
        let fn_name = item_fn.sig.ident.clone();

        Ok(Self {
            controller_ty,
            fn_name,
            item_fn,
            schedule,
        })
    }

    /// Requires `async fn name(&self, uuid: Uuid, scheduler: JobScheduler)`.
    fn validate_method(item_fn: ItemFn) -> Result<ItemFn> {
        if item_fn.sig.asyncness.is_none() {
            Err(Error::custom("a job handler must be `async`").with_span(&item_fn.span()))?;
        }

        let mut inputs = item_fn.sig.inputs.iter();

        let Some(FnArg::Receiver(_)) = inputs.next() else {
            Err(Error::custom("a handler must have a `&self` param").with_span(&item_fn.sig))?
        };

        let Some(ty) = inputs.next() else {
            Err(Error::custom("expected `uuid` as the second param").with_span(&item_fn.sig))?
        };

        shared::expect_type(ty, "Uuid")?;

        let Some(ty) = inputs.next() else {
            Err(Error::custom("expected `scheduler` as the third param").with_span(&item_fn.sig))?
        };

        shared::expect_type(ty, "JobScheduler")?;

        Ok(item_fn)
    }
}

impl Schedule {
    pub fn parse(kind: ScheduleKind, attr: TokenStream) -> Result<Self> {
        match kind {
            ScheduleKind::Cron => {
                let lit: LitStr = syn::parse(attr).map_err(|err| {
                    Error::custom(
                        "`#[cron]` expects a string literal, e.g. #[cron(\"*/5 * * * * *\")]",
                    )
                    .with_span(&err.span())
                })?;

                if lit.value().trim().is_empty() {
                    return Err(
                        Error::custom("cron expression must not be empty").with_span(&lit.span())
                    );
                }

                Ok(Schedule::Cron(lit))
            }

            ScheduleKind::Repeated => {
                let expr: Expr = syn::parse(attr).map_err(|err| {
                    Error::custom("`#[repeated]` expects a duration, e.g. #[repeated(Duration::from_secs(5))]").with_span(&err.span())
                })?;

                Ok(Schedule::Repeated(expr))
            }

            ScheduleKind::Once => {
                let expr: Expr = syn::parse(attr).map_err(|err| {
                    Error::custom(
                        "`#[once]` expects a duration, e.g. #[once(Duration::from_secs(30))]",
                    )
                    .with_span(&err.span())
                })?;

                Ok(Schedule::Once(expr))
            }
        }
    }
}
