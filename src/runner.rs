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

    pub fn execute2<F, M>(&mut self, key: M, func: F, step: Option<Step>) -> Result<()>
    where
        F: Fn() -> Result<()>,
        M: Into<Cow<'a, str>> + Debug,
    {
        let key = key.into();
        debug!("Step {:?}", key);

        if let Some(step) = step {
            if !self.ctx.config().should_run(step) {
                return Ok(());
            }
        }

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

                    let should_retry =
                        self.ctx.config().should_ask_for_retry(step) && should_retry(interrupted, key.as_ref())?;

                    if !should_retry {
                        self.report.push_result(Some((key, false)));
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn execute<F, M>(&mut self, key: M, func: F) -> Result<()>
    where
        F: Fn() -> Result<()>,
        M: Into<Cow<'a, str>> + Debug,
    {
        self.execute2(key, func, None)
    }

    pub fn report(&self) -> &Report {
        &self.report
    }
}
