use core::{
    future::{self, IntoFuture},
    marker::PhantomPinned,
};

/// A cancellation token can receive cancellation signal.
pub trait TrCancellationToken {
    /// Test whether this token has received cancellation signal or not.
    fn is_cancelled(&self) -> bool;

    /// Test whether this token will receive cancellation signal or not.
    fn can_be_cancelled(&self) -> bool;

    /// Create a new token that will receive cancellation signal when this
    /// token receives the signal.
    fn child_token(&self) -> impl TrCancellationToken;

    /// Create a future that will become ready when the cancellation signal is
    /// received by this token.
    fn cancellation(&mut self) -> impl IntoFuture;
}

/// A token that is already cancelled and will no never reset.
#[derive(Debug, Default, Clone, Copy)]
pub struct CancelledToken(PhantomPinned);

impl CancelledToken {
    /// Create an instance of `CancelledToken`
    pub const fn new() -> Self {
        CancelledToken(PhantomPinned)
    }

    /// Always true
    pub const fn is_cancelled(&self) -> bool {
        true
    }
    /// Always false
    pub const fn can_be_cancelled(&self) -> bool {
        false
    }

    pub const fn child_token(&self) -> CancelledToken {
        CancelledToken::new()
    }

    /// Always return a ready future.
    pub fn cancellation(&mut self) -> future::Ready<()> {
        future::ready(())
    }
}

impl TrCancellationToken for CancelledToken {
    #[inline]
    fn is_cancelled(&self) -> bool {
        CancelledToken::is_cancelled(self)
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        CancelledToken::can_be_cancelled(self)
    }

    #[inline]
    fn child_token(&self) -> impl TrCancellationToken {
        CancelledToken::child_token(self)
    }

    #[inline]
    fn cancellation(&mut self) -> impl IntoFuture {
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

    /// Always false
    pub const fn is_cancelled(&self) -> bool {
        false
    }

    /// Always false
    pub const fn can_be_cancelled(&self) -> bool {
        false
    }

    pub const fn child_token(&self) -> NonCancellableToken {
        NonCancellableToken::new()
    }

    /// Always returns a pending future.
    pub fn cancellation(&mut self) -> future::Pending<()> {
        future::pending()
    }
}

impl TrCancellationToken for NonCancellableToken {
    #[inline]
    fn is_cancelled(&self) -> bool {
        NonCancellableToken::is_cancelled(self)
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        NonCancellableToken::can_be_cancelled(self)
    }

    #[inline]
    fn child_token(&self) -> impl TrCancellationToken {
        NonCancellableToken::child_token(self)
    }

    #[inline]
    fn cancellation(&mut self) -> impl IntoFuture {
        NonCancellableToken::cancellation(self)
    }
}
