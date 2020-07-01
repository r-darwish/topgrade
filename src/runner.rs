use crate::ctrlc;
use crate::error::SkipStep;
use crate::execution_context::ExecutionContext;
use crate::report::Report;
use crate::{config::Step, terminal::should_retry};
use anyhow::Result;
use log::debug;
use std::borrow::Cow;
use std::fmt::Debug;

pub struct Runner<'a> {
    ctx: &'a ExecutionContext<'a>,
    report: Report<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(ctx: &'a ExecutionContext) -> Runner<'a> {
        Runner {
            ctx,
            report: Report::new(),
        }
    }

    pub fn execute<F, M>(&mut self, step: Step, key: M, func: F) -> Result<()>
    where
        F: Fn() -> Result<()>,
        M: Into<Cow<'a, str>> + Debug,
    {
        if !self.ctx.config().should_run(step) {
            return Ok(());
        }

        let key = key.into();
        debug!("Step {:?}", key);

        loop {
            match func() {
                Ok(()) => {
                    self.report.push_result(Some((key, true)));
                    break;
                }
                Err(e) if e.downcast_ref::<SkipStep>().is_some() => {
                    break;
                }
                Err(_) => {
                    let interrupted = ctrlc::interrupted();
                    if interrupted {
                        ctrlc::unset_interrupted();
                    }

                    let should_ask = interrupted || !self.ctx.config().no_retry();
                    let should_retry = should_ask && should_retry(interrupted, key.as_ref())?;

                    if !should_retry {
                        self.report.push_result(Some((key, false)));
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn report(&self) -> &Report {
        &self.report
    }
}
