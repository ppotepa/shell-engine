# engine-io

Transport-agnostic IPC bridge for sidecar communication.

## Purpose

Defines a trait-based transport layer for communicating with external
sidecar processes (e.g., the C# cognitOS simulator). Supports TCP
connections, process lifecycle management, and a null transport for
testing without a live sidecar.

## Key Types

- `SidecarTransport` — trait for sending/receiving `IpcMessage` values
- `TcpSidecar` — TCP-based transport implementation
- `SidecarProcess` — manages spawning and stopping the sidecar process
- `NullTransport` — no-op transport for headless and test runs
- `IpcMessage` — JSON-serialized message envelope

## Dependencies

- `serde` / `serde_json` — message serialization
- `thiserror` — error type derivation

## Usage

The runtime selects a transport at startup. Scene behaviors send
commands to the sidecar via `SidecarTransport::send()` and poll
responses with `recv()`.
