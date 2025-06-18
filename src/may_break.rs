use core::ops::{ControlFlow, Try};

use crate::cancellation::{NonCancellableToken, TrCancellationToken};

/// Describes a coroutine, a loop, or a job to run on other thread, that can be
/// discontinued with an external cancellation token.
/// 
/// ## Usage Note
/// 
/// * This is different from `TrMayCancel` in that it does not work with
///   async/await. No asynchronous runtime is involved, nor futures;
/// * This is assumed to work only that, the coroutine, or loop, or the job,
///   actively or passively poll the cancellation token for signal;
pub trait TrMayBreak: Sized {
    type MayBreakOutput: Sized;

    fn may_break_with<C>(self, cancel: &mut C) -> Self::MayBreakOutput
    where
        C: TrCancellationToken;

    fn wait(self) -> Self::MayBreakOutput {
        self.may_break_with(NonCancellableToken::shared_mut())
    }

    fn wait_and_unwrap(self) -> <Self::MayBreakOutput as Try>::Output
    where
        Self::MayBreakOutput: Try,
    {
        let ControlFlow::Continue(x) = self.wait().branch() else {
            panic!()
        };
        x
    }
}

pub struct Completed<T>(T);

impl<T> Completed<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn may_break_with<C>(self, _: &mut C) -> T
    where
        C: TrCancellationToken,
    {
        self.0
    }
}

impl<T> From<T> for Completed<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> TrMayBreak for Completed<T> {
    type MayBreakOutput = T;

    #[inline]
    fn may_break_with<C>(self, cancel: &mut C) -> Self::MayBreakOutput
    where
        C: TrCancellationToken,
    {
        Completed::may_break_with(self, cancel)
    }
}
