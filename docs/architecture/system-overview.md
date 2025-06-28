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
