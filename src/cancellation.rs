use core::{
    cell::SyncUnsafeCell,
    future::{self, Future, IntoFuture},
    marker::PhantomPinned,
    pin::Pin,
};

pub trait TrCancellationToken: Clone {
    type Cancellation<'a>: 'a + IntoFuture<Output = ()> where Self: 'a;

    fn is_cancelled(&self) -> bool;

    fn can_be_cancelled(&self) -> bool;

    fn cancellation(self: Pin<&mut Self>) -> Self::Cancellation<'_>;
}

pub trait TrConfigCancelSignal: IntoFuture<Output = ()> {
    /// Configure the future of cancellation signal should turn ready when the 
    /// cancellation token is orphaned.
    fn cancel_on_orphaned(self) -> impl IntoFuture<Output = ()>;

    /// Configure the future of cancellation signal should stay pending even 
    /// though the cancellation token is orphaned.
    fn pend_on_orphaned(self) -> impl IntoFuture<Output = ()>;
}

pub trait TrIntoFutureMayCancel<'a>
where
    Self: 'a + Sized,
{
    type MayCancelOutput;

    fn may_cancel_with<C>(
        self,
        cancel: Pin<&'a mut C>,
    ) -> impl Future<Output = Self::MayCancelOutput>
    where
        C: TrCancellationToken;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CancelledToken(PhantomPinned);

impl CancelledToken {
    pub const fn new() -> Self {
        CancelledToken(PhantomPinned)
    }

    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<CancelledToken> =
            SyncUnsafeCell::new(CancelledToken::new());
        unsafe {
            #[allow(static_mut_refs)]
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    pub fn cancellation(self: Pin<&mut Self>) -> future::Ready<()> {
        future::ready(())
    }
}

impl TrCancellationToken for CancelledToken {
    type Cancellation<'a> = future::Ready<()> where Self: 'a;

    fn is_cancelled(&self) -> bool {
        true
    }

    fn can_be_cancelled(&self) -> bool {
        false
    }

    fn cancellation(self: Pin<&mut Self>) -> Self::Cancellation<'_> {
        CancelledToken::cancellation(self)
    }
}

/// A cancellation token that will never be cancelled, usually used
/// as a dummy for `TrCancellationToken`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NonCancellableToken(PhantomPinned);

impl NonCancellableToken {
    pub const fn new() -> Self {
        NonCancellableToken(PhantomPinned)
    }

    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<NonCancellableToken> =
            SyncUnsafeCell::new(NonCancellableToken::new());
        unsafe {
            #[allow(static_mut_refs)]
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    pub fn cancellation(self: Pin<&mut Self>) -> future::Pending<()> {
        future::pending()
    }
}

impl TrCancellationToken for NonCancellableToken {
    type Cancellation<'a> = future::Pending<()> where Self: 'a;

    fn is_cancelled(&self) -> bool {
        false
    }

    fn can_be_cancelled(&self) -> bool {
        false
    }

    fn cancellation(self: Pin<&mut Self>) -> Self::Cancellation<'_> {
        NonCancellableToken::cancellation(self)
    }
}
