# engine-io

Transport-agnostic sidecar bridge for external process integration.

## Purpose

`engine-io` defines the JSON-line protocol and transport abstractions used to
talk to sidecar applications such as the `cognitOS` simulator.

It supports both stdio-backed child processes and localhost TCP sidecars behind
the same `SidecarTransport` contract.

## Key types

- `IoRequest` — outbound requests sent to the sidecar
- `IoEvent` — inbound sidecar events consumed by the engine
- `SidecarTransport` — common transport trait
- `SidecarProcess` — stdio transport/process host
- `TcpSidecar` — TCP transport/process host
- `EngineIoError` — transport/process error type

## Working with this crate

- keep the protocol transport-agnostic and scene-agnostic,
- preserve JSON compatibility because engine and sidecar may evolve independently,
- when adding new request/event variants, update both the engine handlers and the sidecar implementation together.
