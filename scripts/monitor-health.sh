#!/bin/bash
# Monitor running Petra instance health

echo "📊 Petra Health Monitor"
echo "====================="

# Check process
PID=$(pgrep petra)
if [ -n "$PID" ]; then
    echo "✅ Petra running (PID: $PID)"
    
    # Memory usage
    ps -p $PID -o %mem,rss,vsz
    
    # Open file descriptors
    lsof -p $PID | wc -l
    
    # Thread count
    ps -p $PID -o nlwp
else
    echo "❌ Petra not running"
fi

# Check logs for errors
tail -n 100 /var/log/petra/petra.log | grep -E "ERROR|PANIC" || echo "✅ No recent errors"
