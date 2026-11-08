mod expand;
mod parsing;

use proc_macro::TokenStream;

use self::parsing::ParsedScheduleHandler;
use crate::shared::Result;

pub use self::parsing::ScheduleKind;

pub fn expand_job_task(
    kind: ScheduleKind,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream> {
    let parsed = ParsedScheduleHandler::parse(&kind, attr, item)?;
    Ok(expand::expand_schedule_handler(parsed)?.into())
}
