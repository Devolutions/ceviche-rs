# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [[0.8.0](https://github.com/Devolutions/ceviche-rs/compare/v0.7.0...v0.8.0)] - 2026-10-13

### <!-- 1 -->Features

- [**breaking**] Propagate service exit code to the caller ([#36](https://github.com/Devolutions/ceviche-rs/issues/36)) ([0786fea178](https://github.com/Devolutions/ceviche-rs/commit/0786fea17840ac80013e2615cdea1793058929f3)) 

  `ServiceMainFn<T>` now returns `u8`, matching POSIX's 0-255 exit code
  range (0 = success, 1-255 = failure).
  On all platforms, `Controller::register` now returns `Result<u8, Error>`
  with the exit code of the service main function, so callers can inspect
  it and propagate it to the OS (e.g. `fn main() -> ExitCode` +
  `ExitCode::from(code)`).
  
  On Linux/macOS, `dispatch` and the `Service!` macro wrapper return `u8`,
  which `register` forwards to the caller.
  
  On Windows, the `Service!` wrapper must match the
  `LPSERVICE_MAIN_FUNCTIONW` ABI and cannot return a value. Instead,
  `dispatch`:
  - reports the exit code to the SCM via `SetServiceStatus` with
  `dwWin32ExitCode = ERROR_SERVICE_SPECIFIC_ERROR` and
  `dwServiceSpecificExitCode = u32::from(exit_code)` (or 0/0 on success);
  - stores it before reporting `SERVICE_STOPPED`, so `register` returns it
  once `StartServiceCtrlDispatcherW` unblocks.
  
  Also on Windows, no controls are accepted while in
  `SERVICE_START_PENDING` or `SERVICE_STOPPED`.
  
  The crate docs, the `foobar` example and the `cmdlet` are updated to the
  new API and propagate the exit code.

### <!-- 4 -->Bug Fixes

- Terminate the service table passed to StartServiceCtrlDispatcherW ([#37](https://github.com/Devolutions/ceviche-rs/issues/37)) ([ba51058d70](https://github.com/Devolutions/ceviche-rs/commit/ba51058d701436964a09d7a5f9b6e95e58f55dfa)) 

  Fixes a crash at Windows service startup.

- [**breaking**] Harden service status reporting ([#39](https://github.com/Devolutions/ceviche-rs/issues/39)) ([28de845fe1](https://github.com/Devolutions/ceviche-rs/commit/28de845fe1548beadc9da758af987a0feabe6426)) 

  The control handler read a `static mut` status handle that `dispatch`
  wrote only after registering the handler, so an early control could race
  with the write or see an unset handle. The handler context also pointed
  to a sender on the stack of `dispatch`. STOP_PENDING was reported with a
  10 ms wait hint and a zero checkpoint.
  
  The handler now receives a context that is never freed, holding the
  event sender and a mutex-protected status handle. `dispatch` holds the
  lock across registration, so the handler waits for the handle instead of
  seeing it unset. Status updates after SERVICE_STOPPED are dropped, so a
  late stop control cannot report STOP_PENDING after STOPPED.
  
  Pending states now report a realistic wait hint (5 s for START_PENDING,
  30 s for STOP_PENDING) with an incrementing checkpoint, and other states
  report zero for both. STOP_PENDING is reported once instead of being
  renewed on a timer: the service cannot report shutdown progress, and a
  timer-driven checkpoint would hide a hung shutdown. STOP_PENDING also
  accepts no controls.


