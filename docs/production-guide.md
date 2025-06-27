# Petra Production Guide
> Hardened deployment checklist for Petra Automation Engine  
---

## Table of Contents
1. [Prerequisites](#prerequisites)  
2. [CPU Isolation](#cpu-isolation)  
3. [Systemd Service](#systemd-service)  
4. [Monitoring with Prometheus](#monitoring-with-prometheus)  
5. [Backup & Recovery](#backup--recovery)  
6. [Troubleshooting](#troubleshooting)  
7. [Security Hardening](#security-hardening)  
8. [Disaster Recovery & Multi-Site](#disaster-recovery--multi-site)  
9. [Repository Layout](#repository-layout)  

---

## Prerequisites
* Petra binaries installed to **`/opt/petra/bin/`**  
* User & group **`petra`** created  
* Root shell or **`sudo`** privileges  

---

## CPU Isolation
1. Edit **`/etc/default/grub`**:

   ```bash
   GRUB_CMDLINE_LINUX="isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3 irqaffinity=0,1"
   sudo update-grub
   sudo reboot
````

2. Verify:

   ```bash
   grep -o 'isolcpus=[^ ]*' /proc/cmdline
   ```

---

## Systemd Service

Copy `examples/systemd/petra.service` into **`/etc/systemd/system/`** (or paste below):

```ini
[Unit]
Description=Petra Automation Engine
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
ExecStart=/opt/petra/bin/petra \
  --config /etc/petra/config.yaml \
  --rt-priority 50 \
  --cpu-affinity 2,3 \
  --lock-memory
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10

# Security
User=petra
Group=petra
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/petra /var/log/petra

# Resource limits
LimitNOFILE=1048576
LimitMEMLOCK=infinity
LimitRTPRIO=99
CPUAffinity=2 3
Nice=-10

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=petra

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now petra
```

---

## Monitoring with Prometheus

*Example files live in `examples/prometheus/`.*

1. **`prometheus.yml`**

   ```yaml
   global:
     scrape_interval: 15s
     evaluation_interval: 15s
   # …
   ```

2. **`petra_rules.yml`**

   ```yaml
   groups:
     - name: petra_alerts
       rules:
         - alert: PetraHighErrorRate
           expr: rate(petra_engine_errors_total[5m]) > 0.01
           # …
   ```

3. Reload Prometheus:

   ```bash
   sudo systemctl reload prometheus
   ```

---

## Backup & Recovery

Scripts are in `scripts/` for version control.

### Automated Backup

`/opt/petra/scripts/backup.sh`

```bash
set -euo pipefail
# …
```

Schedule it via **`cron`** or **`systemd-timer`**.

### Recovery Procedure

`/opt/petra/scripts/restore.sh`

```bash
#!/bin/bash
# restore.sh <backup_file.tar.gz>
# …
```

---

## Troubleshooting

Common commands:

```bash
# CPU
top -H -p $(pidof petra)

# Scan metrics
curl -s http://localhost:9090/metrics | grep petra_engine_scan

# Flame graph (requires cargo-flamegraph)
flamegraph --pid $(pidof petra) -o petra-flame.svg
```

See the full [Troubleshooting Section](#troubleshooting) for CPU, memory, storage tips, and enabling debug logs.

---

## Security Hardening

### AppArmor

Profile example: `security/apparmor/petra`

### SELinux

Module example: `security/selinux/petra.te`

Load the profile:

```bash
sudo apparmor_parser -r security/apparmor/petra
# or
sudo semodule -i security/selinux/petra.pp
```

---

## Disaster Recovery & Multi-Site

Sample S3 replication configs are in `examples/dr/`.
Automated fail-over script: `scripts/failover.sh`

---

## Repository Layout

```
.
├── docs/
│   └── production-guide.md      ← you are here
├── examples/
│   ├── systemd/petra.service
│   ├── prometheus/{prometheus.yml,petra_rules.yml}
│   └── dr/…
├── scripts/
│   ├── backup.sh
│   ├── restore.sh
│   └── failover.sh
└── security/
    ├── apparmor/petra
    └── selinux/petra.te
```
