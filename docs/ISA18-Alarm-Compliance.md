# ISA-18.2 Alarm Management Compliance Guide for PETRA

## Overview

This guide ensures your PETRA alarm notification system complies with ISA-18.2-2016 "Management of Alarm Systems for the Process Industries". The standard provides a framework for designing, implementing, and maintaining effective alarm systems.

## Key ISA-18.2 Requirements & PETRA Implementation

### 1. Alarm Philosophy (Section 6)

**Requirement**: Document alarm system design principles and performance targets.

**PETRA Implementation**:
```yaml
# alarm_philosophy.yaml
alarm_philosophy:
  purpose: "Notify operators of abnormal situations requiring action"
  
  performance_targets:
    average_alarms_per_hour: 6      # Per operator position
    peak_alarms_per_10min: 10       # Flood threshold
    max_standing_alarms: 5          # Active > 24 hours
    
  design_principles:
    - "Every alarm requires operator action"
    - "Alarms must be understandable and actionable"
    - "Similar alarms use consistent settings"
    - "Nuisance alarms are eliminated"
```

### 2. Alarm Identification (Section 7.2)

**Requirement**: Each alarm must have unique identification and documentation.

**PETRA Implementation**:
```rust
pub struct AlarmConfig {
    // Unique identifier (7.2.1)
    pub name: String,              // e.g., "PT_101_HH"
    
    // Documentation (7.2.2)
    pub description: String,       // Clear description
    pub tag_name: String,          // Process tag
    pub consequence: String,       // What happens if ignored
    pub corrective_action: String, // Required operator response
    pub max_response_time: Option<u32>, // Minutes
}
```

### 3. Alarm Prioritization (Section 6.5)

**Requirement**: Assign priorities based on consequence severity and response time.

**PETRA Implementation**:
```rust
pub enum AlarmPriority {
    Critical = 1,  // Immediate action (0-3 minutes)
    High = 2,      // Prompt action (3-10 minutes)  
    Medium = 3,    // Action required (10-30 minutes)
    Low = 4,       // Awareness (30+ minutes)
}
```

**Priority Distribution Target** (per ISA-18.2):
- Critical: ~5% of configured alarms
- High: ~15% of configured alarms
- Medium: ~80% of configured alarms
- Low: Very few (journal/event only)

### 4. Alarm States (Section 7.3)

**Requirement**: Implement proper alarm state machine.

**PETRA Implementation**:
```rust
pub enum AlarmState {
    Normal,                        // No alarm condition
    Unacknowledged,               // Active, not acknowledged
    Acknowledged,                  // Active and acknowledged
    ReturnToNormalUnacknowledged, // Cleared but not acknowledged
    Suppressed,                    // By design suppression
    OutOfService,                  // Maintenance mode
    Shelved,                       // Operator suppression
}
```

**State Transition Rules**:
```
Normal → Unacknowledged (alarm activates)
Unacknowledged → Acknowledged (operator acknowledges)
Unacknowledged → RTN-Unack (alarm clears)
Acknowledged → Normal (alarm clears)
RTN-Unack → Normal (operator acknowledges)
```

### 5. Alarm Display Requirements (Section 8)

**HMI Display Requirements**:

1. **Color Coding** (per ISA-18.2 and ANSI/ISA-101):
   - Critical: Red (#FF0000) on white
   - High: Orange (#FF8C00) on white
   - Medium: Yellow (#FFFF00) on black
   - Low: Cyan (#00FFFF) on black

2. **Blinking**: Only for unacknowledged alarms (0.5-2 Hz)

3. **Required Information**:
   - Tag name
   - Description
   - Priority
   - Current value and units
   - Setpoint/limit
   - Time in alarm
   - Area/equipment

4. **Alarm List Sorting**:
   - Primary: Priority (Critical → Low)
   - Secondary: Time (newest first)
   - User-selectable: Area, equipment, acknowledge state

### 6. Alarm Response Procedures (Section 7.4)

**Requirement**: Document operator response for each alarm.

**PETRA Configuration**:
```yaml
alarms:
  - name: "PT_101_HH"
    corrective_action: |
      1. Open PV-101 to 100% immediately
      2. Reduce feed rate to minimum
      3. If pressure continues rising:
         - Initiate emergency shutdown
         - Evacuate area
      4. Notify supervisor
    reference_documents:
      - "EOP-101: Emergency Shutdown Procedure"
      - "P&ID-100: Reactor System"
```

### 7. Alarm Suppression (Section 9.4)

**Design Suppression Types**:

1. **State-Based Suppression**:
```rust
// Suppress low flow alarm when pump is off
if pump_state == "stopped" {
    suppress_alarm("FT_301_L");
}
```

2. **First-Out Suppression**:
```yaml
suppression_groups:
  reactor_trip:
    first_out: "PT_101_HH"  # Only show initiating alarm
    suppress: ["TT_102_H", "FT_301_L", "LT_201_L"]
```

3. **Flood Suppression**:
```rust
if alarms_per_10min > 10 {
    suppress_priorities(vec![AlarmPriority::Low]);
}
```

### 8. Alarm Shelving (Section 9.5)

**Operator Suppression Rules**:
- Maximum shelve duration: 24 hours
- Requires authorization
- Must be logged with reason
- Automatic unshelving

```rust
pub async fn shelve_alarm(&mut self, 
    name: &str, 
    user: &str, 
    duration_minutes: u32,
    reason: &str
) -> Result<()> {
    // Validate authorization
    if !user_authorized(user, "shelve_alarm") {
        return Err("Unauthorized");
    }
    
    // Maximum 24 hours
    if duration_minutes > 1440 {
        return Err("Exceeds maximum shelve duration");
    }
    
    // Log the action
    audit_log(format!(
        "{} shelved {} for {} minutes: {}", 
        user, name, duration_minutes, reason
    ));
}
```

### 9. Performance Monitoring (Section 10)

**Required Metrics**:

1. **Alarm Rate Metrics**:
   - Average alarms per hour
   - Peak alarms per 10 minutes
   - Time in flood condition

2. **Standing Alarm Metrics**:
   - Alarms active > 24 hours
   - Alarms active > 7 days

3. **Acknowledgment Metrics**:
   - Average time to acknowledge
   - Unacknowledged alarm count

4. **Frequently Occurring Alarms**:
   - Top 10 bad actors
   - Activation count per alarm

**PETRA Implementation**:
```rust
#[derive(Serialize)]
pub struct AlarmMetrics {
    pub hourly_rate: Vec<f64>,        // Last 24 hours
    pub current_active: usize,
    pub standing_alarms: Vec<String>, // Active > 24h
    pub bad_actors: Vec<(String, u64)>, // Top 10
    pub avg_ack_time: Duration,
}
```

### 10. Alarm Documentation (Section 11)

**Required Documentation Per Alarm**:

1. **Alarm Database Record**:
```yaml
alarm_database:
  PT_101_HH:
    tag: "PT-101"
    description: "Reactor Pressure High-High"
    setpoint: 250.0
    units: "PSIG"
    priority: "Critical"
    classification: "Safety"
    
    design_basis: |
      Setpoint at 80% of vessel MAWP (300 PSIG)
      Safety margin for emergency response
      
    consequence: |
      Vessel rupture, potential explosion
      Personnel injury/fatality risk
      Environmental release
      
    corrective_action: |
      1. Open PV-101 immediately
      2. Stop feed pumps
      3. Initiate emergency shutdown if needed
      
    verification_method: |
      Simulate with pressure calibrator
      Response time < 2 seconds
      
    references:
      - "PSV-101 sizing calculation"
      - "HAZOP Node 3, Deviation 2"
      - "Layer of Protection Analysis"
```

### 11. Testing and Maintenance (Section 12)

**Testing Requirements**:

1. **Initial Testing**:
   - Verify alarm activates at setpoint
   - Confirm correct priority/classification
   - Test acknowledgment function
   - Verify notification routing

2. **Periodic Testing**:
```yaml
maintenance_schedule:
  critical_alarms:
    frequency: "Monthly"
    procedure: "Inject test signal, verify response"
    
  high_alarms:
    frequency: "Quarterly"
    procedure: "Functional test with simulator"
    
  all_alarms:
    frequency: "Annual"
    procedure: "Full loop test including final elements"
```

### 12. Management of Change (Section 13)

**Change Control Process**:

1. **Alarm Addition Checklist**:
   - [ ] Justified need (not just "nice to have")
   - [ ] Unique condition (no duplicates)
   - [ ] Correct priority assigned
   - [ ] Operator can take action
   - [ ] Response procedure documented
   - [ ] Added to training materials
   - [ ] Tested before implementation

2. **Alarm Modification Log**:
```rust
#[derive(Serialize)]
pub struct AlarmChange {
    pub timestamp: DateTime<Utc>,
    pub alarm_name: String,
    pub change_type: String, // Added/Modified/Removed
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
    pub approved_by: String,
    pub tested_by: String,
}
```

## Implementation Checklist

### Phase 1: Foundation
- [ ] Create alarm philosophy document
- [ ] Implement ISA-18.2 data structures
- [ ] Add alarm state machine
- [ ] Create unique alarm identifiers

### Phase 2: Core Features
- [ ] Implement 4-level priority system
- [ ] Add acknowledgment functionality
- [ ] Create alarm history/logging
- [ ] Build HMI display per ISA-101

### Phase 3: Advanced Features
- [ ] Add alarm shelving
- [ ] Implement suppression logic
- [ ] Create flood detection
- [ ] Add performance metrics

### Phase 4: Documentation
- [ ] Document all alarms in database
- [ ] Create response procedures
- [ ] Develop training materials
- [ ] Establish testing procedures

### Phase 5: Continuous Improvement
- [ ] Monitor performance metrics
- [ ] Identify and eliminate bad actors
- [ ] Regular alarm rationalization
- [ ] Periodic audit against ISA-18.2

## Configuration Example

```yaml
# Complete ISA-18.2 compliant alarm configuration
alarms:
  - name: "PT_101_HH"
    description: "Reactor Pressure High-High"
    tag_name: "PT-101"
    signal: "reactor.pressure"
    
    # Condition
    condition:
      type: "HighHigh"
      threshold: 250.0
    hysteresis: 5.0
    
    # ISA-18.2 Required Fields
    priority: "Critical"
    classification: "Safety"
    consequence: "Vessel rupture risk"
    corrective_action: "Open PV-101, initiate ESD if needed"
    max_response_time: 3
    
    # Location
    area: "Reactor Area"
    equipment: "R-101"
    
    # Settings
    enabled: true
    units: "PSIG"
    on_delay_ms: 0
    off_delay_ms: 5000
    
    # Actions
    actions:
      - type: "Signal"
        name: "emergency_shutdown"
        value: true
      - type: "Email"
        recipients: ["control-room@plant.com"]

# Performance targets
performance:
  targets:
    avg_alarm_rate: 6
    peak_alarm_rate: 10
    max_standing: 5
    
  monitoring:
    enabled: true
    report_frequency: "daily"
    bad_actor_threshold: 10  # Alarms/day
```

## Compliance Verification

Use this checklist to verify ISA-18.2 compliance:

1. **Documentation**:
   - [ ] Alarm philosophy exists and is current
   - [ ] All alarms have required documentation
   - [ ] Response procedures are available

2. **Design**:
   - [ ] Priority distribution follows guidelines
   - [ ] Alarm states match ISA-18.2
   - [ ] Each alarm requires action

3. **Display**:
   - [ ] Colors match standards
   - [ ] Required information shown
   - [ ] Acknowledgment available

4. **Performance**:
   - [ ] Metrics are tracked
   - [ ] Targets are met
   - [ ] Bad actors addressed

5. **Management**:
   - [ ] Change control in place
   - [ ] Testing performed
   - [ ] Training provided
