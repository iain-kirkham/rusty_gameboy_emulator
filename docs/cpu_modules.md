# CPU Module Layout

This project splits CPU internals across focused files under `src/cpu/` while keeping `CPU` behavior unchanged.

- `src/cpu.rs`: CPU state (`CPU` struct), high-level step orchestration, and public entry points.
- `src/cpu/execute_impl.rs`: opcode-family execution dispatch and per-family handlers.
- `src/cpu/control_flow_impl.rs`: immediate reads and branch/call/return address calculations.
- `src/cpu/stack_interrupt_impl.rs`: stack push/pop helpers and interrupt service flow.
- `src/cpu/load_impl.rs`: `LD` byte operand reads/writes and byte-load timing/PC increment helpers.
- `src/cpu/arithmetic_impl.rs`: arithmetic/logic data-path helpers and flag update routines.
- `src/cpu/fetch_trace_impl.rs`: instruction fetch/decode and debug trace output.
- `src/cpu/prefix_impl.rs`: CB-prefixed register/(HL) operand access helpers.

## Visibility Rule

Prefer `pub(super)` for helper methods shared between CPU submodules. Keep methods private when used only inside one file. Keep `pub(crate)` only for external CPU API (`new`, `step`, `is_halted`).

## Behavioral Invariants

- Preserve interrupt ordering and service sequence exactly (`IME` disable -> stack push PC -> clear interrupt -> jump).
- Preserve HALT and HALT-bug behavior exactly.
- Preserve EI delayed-enable timing.
- Preserve stack byte order (push high then low; pop low then high).
- Preserve per-instruction cycle counts and PC advancement rules.

