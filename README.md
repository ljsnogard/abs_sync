# abs_sync

Abstraction of synchronization for Rust async/await.  
This crate provide traits about locks in sync and/or async environment.  

## Required unstable features:

```
#![feature(sync_unsafe_cell)]
#![feature(try_trait_v2)]
#![feature(type_alias_impl_trait)]
```

## Example

```rust
use core::{
    borrow::BorrowMut,
    ops::{ControlFlow, Deref, DerefMut},
};

use pin_utils::pin_mut;
use abs_sync::{
    cancellation::{NonCancellableToken, TrIntoFutureMayCancel},
    x_deps::pin_utils,
};

async fn demo<B, L, T>(rwlock: B)
where
    B: BorrowMut<L>,
    L: TrAsyncRwLock<Target = T>,
{
    let acq = rwlock.borrow().acquire();
    pin_mut!(acq);
    let read_async = acq.as_mut().read_async();
    // let write_async = acq.write_async(); // illegal
    let ControlFlow::Continue(read_guard) = read_async
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = read_guard.deref();
    // let write_async = acq.write_async(); // illegal
    drop(read_guard);
    let ControlFlow::Continue(upgradable) = acq
        .as_mut()
        .upgradable_read_async()
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = upgradable.deref();
    let upgrade = upgradable.upgrade();
    pin_mut!(upgrade);
    let ControlFlow::Continue(mut write_guard) = upgrade
        .upgrade_async()
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = write_guard.deref_mut();
    let upgradable = write_guard.downgrade_to_upgradable();
    drop(upgradable)
}
```

## Why?

With considered API design comes with `abs_sync`, a better pattern that reduces deadlock can apply to daily lock acquiring situations.  

You may want to read [Rust's Sneaky Deadlock With `if let` Blocks](https://brooksblog.bearblog.dev/rusts-sneaky-deadlock-with-if-let-blocks/).

With `abs_sync`, the contenders are forced to acquire the lock with their own `Acquire` instances.  
That means, compiler can stop you from dead-lock:

```rust

async fn prevent_dead_lock_demo<B, L, T>(
    rwlock: &'static impl TrAsyncRwLock<Target = T>,
) {
    let acq = rwlock.acquire();
    pin_mut!(acq);

    // acq first mutable borrow occurs here
    let branch_read = acq
        .as_mut()
        .read_async()
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch();
    if let ControlFlow::Continue(_read) = branch_read {

    } else {
        // cannot borrow `acq` as mutable more than once at a time
        let branch_write = acq
            .as_mut()
            .write_async()
            .may_cancel_with(NonCancellableToken::pinned())
            .await
            .branch();
        let ControlFlow::Continue(_write) = branch_write else {
            unreachable!()
        };
    }
}

```