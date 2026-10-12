mod expand;
mod parsing;

use crate::shared::Result;
use parsing::ParsedScheduleHandler;
pub use parsing::ScheduleKind;

use proc_macro::TokenStream;

pub fn expand_job_task(
    kind: ScheduleKind,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream> {
    let parsed = ParsedScheduleHandler::parse(&kind, attr, item)?;
    Ok(expand::expand_schedule_handler(parsed)?.into())
}
