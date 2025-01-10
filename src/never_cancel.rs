use core::{
    future::{Future, IntoFuture},
    mem::MaybeUninit,
    ops::{AsyncFnOnce, Try},
};

use crate::cancellation::{NonCancellableToken, TrIntoFutureMayCancel};

/// An convenient future to implement `IntoFuture` for `TrIntoFutureMayCancel`
/// that will not actually being cancelled, and that the output will differs
/// (but deducible) from the cancellable version.
pub struct FutureForTaskNeverCancel<T>
where
    T: TrIntoFutureMayCancel<MayCancelOutput: Try>,
{
    task_: MaybeUninit<T>,
}

impl<T> FutureForTaskNeverCancel<T>
where
    T: TrIntoFutureMayCancel<MayCancelOutput: Try>,
{
    pub fn new(task: T) -> Self {
        FutureForTaskNeverCancel {
            task_: MaybeUninit::new(task),
        }
    }

    async fn run_without_cancel(self) -> T::MayCancelOutput {
        let task = unsafe { self.task_.assume_init_read() };
        let cancel = NonCancellableToken::pinned();
        task.may_cancel_with(cancel).await
    }
}

impl<T> AsyncFnOnce<()> for FutureForTaskNeverCancel<T>
where
    T: TrIntoFutureMayCancel<MayCancelOutput: Try>,
{
    type CallOnceFuture = impl Future<Output = Self::Output>;
    type Output = T::MayCancelOutput;

    extern "rust-call" fn async_call_once(
        self,
        _: (),
    ) -> Self::CallOnceFuture {
        self.run_without_cancel()
    }
}

impl<T> IntoFuture for FutureForTaskNeverCancel<T>
where
    T: TrIntoFutureMayCancel<MayCancelOutput: Try>,
{
    type IntoFuture = <Self as AsyncFnOnce<()>>::CallOnceFuture;
    type Output = <Self::IntoFuture as Future>::Output;

    fn into_future(self) -> Self::IntoFuture {
        self()
    }
}
