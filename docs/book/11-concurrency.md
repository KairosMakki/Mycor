# Concurrency

Mycor provides lightweight concurrency primitives. Status: syntax present; channels, mutex, atomics, scoped threads are partially implemented (see `ROADMAP.md`).

## spawn

```myc
spawn {
    println("background work");
}
```

## Channels

```myc
let (tx, rx) = channel[i32]();
spawn { tx.send(42); }
let v = rx.recv();
```

Type mismatches are reported (`E0503`).

## Scoped threads

```myc
scope |s| {
    s.spawn(|| work(&data));
}
```

Scoped threads may borrow data from the enclosing scope safely.

## Send and Sync

- `Send`: safe to transfer between threads.
- `Sync`: safe to share references between threads.

Non-sendable values cannot be sent (`E0501`); non-sync shared access is an error (`E0502`); data races are errors (`E0500`).

## Atomics and mutex (planned)

Atomic types and mutexes are on the roadmap for full standard library support. Current scaffolding lives in `stdlib/`.