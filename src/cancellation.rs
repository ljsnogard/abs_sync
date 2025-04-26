use core::{
    cell::SyncUnsafeCell,
    future::{self, IntoFuture},
    marker::PhantomPinned,
    pin::Pin,
};

pub trait TrCancellationToken: Clone {
    fn is_cancelled(&self) -> bool;

    fn can_be_cancelled(&self) -> bool;

    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()>;
}

/// An instance of [IntoFuture] for an async task that may or may not be
/// cancelled by an optional cancellation token.
///
/// Note: the lifetime here is required by `rustc` when implementing 
/// [TrMayCancel] for your type. Along with future release of rustc, the `<'a>`
/// may be removed.
pub trait TrMayCancel<'a>
where
    Self: 'a + Sized + IntoFuture
{
    type MayCancelOutput;

    fn may_cancel_with<'f, C: TrCancellationToken>(
        self,
        cancel: Pin<&'f mut C>,
    ) -> impl IntoFuture<Output = Self::MayCancelOutput>
    where
        Self: 'f;
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
    #[inline]
    fn is_cancelled(&self) -> bool {
        true
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        false
    }

    #[inline]
    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
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
    #[inline]
    fn is_cancelled(&self) -> bool {
        false
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        false
    }

    #[inline]
    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
        NonCancellableToken::cancellation(self)
    }
}
