# Blocking Operations Fix Analysis

## Issue Fixed
The ALSA POLLERR when running both `start_icecast_thread` and `start_file_save_thread` was caused by blocking operations in the audio callback (`handle_samples` function).

## Root Cause
Audio callbacks must be real-time safe and cannot block. The following blocking operations in `handle_samples` were causing ALSA timeouts:
- `state.blocking_read().clone()` at line ~420
- `state.blocking_write()` at lines ~432 and ~445

## Applied Fix
Replaced blocking operations with non-blocking alternatives:

```rust
// Before:
let read_state = {
    state.blocking_read().clone()
};

// After:
let read_state = match state.try_read() {
    Ok(guard) => guard.clone(),
    Err(_) => return, // Skip if can't acquire lock
};

// Before:
let mut state = state.blocking_write();
(*state).is_audio_present = true;

// After:
if let Ok(mut state) = state.try_write() {
    (*state).is_audio_present = true;
}
```

## Remaining Blocking Operations
Found 6 additional blocking operations in the Icecast thread that could be optimized:

### 1. Streaming Status Updates (lines 282-289)
```rust
// Current:
if !state.blocking_read().is_streaming {
    let mut mutable_state = state.blocking_write();
    mutable_state.is_streaming = true;
    let _ = message_bus.send(AlasMessage::StreamingStarted);
}

// Suggested fix:
if let Ok(read_state) = state.try_read() {
    if !read_state.is_streaming {
        drop(read_state);
        if let Ok(mut mutable_state) = state.try_write() {
            mutable_state.is_streaming = true;
            let _ = message_bus.send(AlasMessage::StreamingStarted);
        }
    }
}
```

### 2. Error Handling (lines 289-292)
```rust
// Current:
let mut mutable_state = state.blocking_write();
mutable_state.is_streaming = false;
let _ = message_bus.send(AlasMessage::StreamingStopped);

// Suggested fix:
if let Ok(mut mutable_state) = state.try_write() {
    mutable_state.is_streaming = false;
    let _ = message_bus.send(AlasMessage::StreamingStopped);
}
```

### 3. Loop Cleanup (lines 312-315)
```rust
// Current:
let mut mutable_state = state.blocking_write();
mutable_state.is_streaming = false;
let _ = message_bus.send(AlasMessage::StreamingStopped);

// Suggested fix:
if let Ok(mut mutable_state) = state.try_write() {
    mutable_state.is_streaming = false;
    let _ = message_bus.send(AlasMessage::StreamingStopped);
}
```

### 4. Thread Exit Cleanup (lines 320-322)
```rust
// Current:
let mut mutable_state = state.blocking_write();
mutable_state.is_streaming = false;
let _ = message_bus.send(AlasMessage::StreamingStopped);

// Suggested fix:
if let Ok(mut mutable_state) = state.try_write() {
    mutable_state.is_streaming = false;
    let _ = message_bus.send(AlasMessage::StreamingStopped);
}
```

### 5. Icecast Connection Setup (line 372)
```rust
// Current:
let state = state.blocking_read();

// Suggested fix:
let state = match state.try_read() {
    Ok(guard) => guard,
    Err(_) => {
        std::thread::sleep(std::time::Duration::from_millis(10));
        continue; // Retry connection attempt
    }
};
```

## Priority
- **Critical (FIXED)**: Audio callback blocking operations - these caused the ALSA POLLERR
- **Medium**: Icecast thread blocking operations - these could cause lock contention but are less critical

## Notes
The audio callback fix was essential for real-time audio processing. The remaining fixes are optimizations that would improve overall system responsiveness but are not critical for basic functionality.