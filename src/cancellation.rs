use core::{
    cell::SyncUnsafeCell,
    future::{self, IntoFuture},
    marker::PhantomPinned,
    pin::Pin,
};

pub trait TrCancellationToken: Clone {
    type Cancellation<'a>: 'a + TrConfigSignal where Self: 'a;

    fn is_cancelled(&self) -> bool;

    fn can_be_cancelled(&self) -> bool;

    fn cancellation(self: Pin<&mut Self>) -> Self::Cancellation<'_>;
}

pub trait TrConfigSignal: IntoFuture<Output = ()> {
    /// Configure the future of cancellation signal should turn ready when the 
    /// cancellation token is orphaned.
    fn cancel_on_orphaned(self) -> impl IntoFuture<Output = ()>;

    /// Configure the future of cancellation signal should stay pending even 
    /// though the cancellation token is orphaned.
    fn pend_on_orphaned(self) -> impl IntoFuture<Output = ()>;
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

impl TrConfigSignal for future::Ready<()> {
    fn cancel_on_orphaned(self) -> impl IntoFuture<Output = ()> {
        self
    }

    fn pend_on_orphaned(self) -> impl IntoFuture<Output = ()> {
        future::pending()
    }
}

impl TrConfigSignal for future::Pending<()> {
    fn cancel_on_orphaned(self) -> impl IntoFuture<Output = ()> {
        self
    }

    fn pend_on_orphaned(self) -> impl IntoFuture<Output = ()> {
        self
    }
}
