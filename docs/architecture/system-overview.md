# Petra System Architecture

## Overview

Petra follows a modular, event-driven architecture designed for reliability, performance, and extensibility.

```mermaid
graph TB
    subgraph "Input Sources"
        S7[S7 PLCs]
        MB[Modbus Devices]
        MQTT1[MQTT Publishers]
    end
    
    subgraph "Petra Core"
        SB[Signal Bus]
        EN[Scan Engine]
        BL[Block Logic]
        AL[Alarm Manager]
        HI[History Manager]
    end
    
    subgraph "Output Destinations"
        MQTT2[MQTT Subscribers]
        TW[Twilio Alerts]
        CH[ClickHouse]
        PQ[Parquet Files]
    end
    
    S7 --> SB
    MB --> SB
    MQTT1 --> SB
    
    SB <--> EN
    EN <--> BL
    BL --> AL
    SB --> HI
    
    AL --> TW
    SB --> MQTT2
    HI --> CH
    HI --> PQ

Core Components
#	Component	Highlights
1	Signal Bus	Central “nervous system” • Thread-safe concurrent access • Event-driven • Hot-path caching
2	Scan Engine	Deterministic cycles (1 ms – 1 s) • Jitter monitoring/compensation • Priority-based execution
3	Block System	Modular logic blocks (15 + built-ins) • Hot-swappable configs • Custom block SDK
4	Storage Layer	Multi-tier design • Local WAL buffering • Remote-storage fail-over • Compression & archival
Data Flow
1 ▪ Input Stage

    Protocol drivers read from field devices.

    Data validated, typed, then written to the Signal Bus.

2 ▪ Processing Stage

    Scan Engine triggers block execution.

    Blocks read inputs from the bus, evaluate logic, and write outputs back.

3 ▪ Output Stage

    MQTT publishers emit changes.

    History Manager persists samples.

    Alarm Manager evaluates conditions and triggers actions (SMS, etc.).

Concurrency Model

    Single scan thread ⇒ deterministic logic execution.

    Async I/O (Tokio) for all protocol drivers.

    Lock-free Signal Bus (DashMap) for low-latency access.

    Buffered channels decouple components and absorb back-pressure.

Reliability Features

    Auto-reconnection for all protocol drivers.

    Local WAL buffers data when offline.

    Graceful degradation when partial connectivity is lost.

    Built-in health checks and metrics endpoints.

Performance Characteristics
Metric	Target	Typical
Scan Time	10 – 100 ms	≈ 50 ms
Scan Jitter	< 10 %	< 5 %
Signal Throughput	100 k / s	≈ 50 k / s
MQTT Messages	10 k / s	≈ 5 k / s
Memory / 1 k Signals	100 MB	≈ 50 MB
Security Architecture

    Authentication required for every connection.

    RBAC authorization model.

    TLS encryption on all network links.

    Audit logging with user context for every action.

    Rigorous input validation & sanitization at every boundary.
