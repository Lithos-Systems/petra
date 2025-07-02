#!/bin/bash
# Comprehensive security review for Petra

set -e

echo "🔒 Petra Security Review"
echo "======================="

# Create security report
REPORT="security_report_$(date +%Y%m%d_%H%M%S).md"

cat > "$REPORT" << EOF
# Security Review Report
Generated: $(date)

## 1. Dependency Audit
EOF

# Check for vulnerable dependencies
echo "### Vulnerable Dependencies" >> "$REPORT"
cargo audit --json | jq -r '.vulnerabilities.list[] | "- \(.advisory.id): \(.advisory.title)"' >> "$REPORT" 2>/dev/null || echo "None found ✓" >> "$REPORT"

# Check for outdated dependencies
echo -e "\n### Outdated Dependencies" >> "$REPORT"
cargo outdated --format json | jq -r '.dependencies[] | select(.outdated) | "- \(.name): \(.version) → \(.latest)"' >> "$REPORT" 2>/dev/null || echo "All up to date ✓" >> "$REPORT"

# Check for unsafe code
echo -e "\n## 2. Unsafe Code Usage" >> "$REPORT"
echo '```' >> "$REPORT"
rg "unsafe" src/ --type rust -A 2 -B 2 >> "$REPORT" || echo "No unsafe code found ✓" >> "$REPORT"
echo '```' >> "$REPORT"

# Check for hardcoded credentials
echo -e "\n## 3. Hardcoded Credentials Check" >> "$REPORT"
PATTERNS=(
    "password.*=.*['\"]"
    "secret.*=.*['\"]"
    "api_key.*=.*['\"]"
    "token.*=.*['\"]"
)

for pattern in "${PATTERNS[@]}"; do
    echo "### Pattern: $pattern" >> "$REPORT"
    rg -i "$pattern" src/ --type rust >> "$REPORT" || echo "None found ✓" >> "$REPORT"
done

# Check TLS configuration
echo -e "\n## 4. TLS Configuration" >> "$REPORT"
echo "### TLS Version Usage" >> "$REPORT"
rg "TLS|tls" src/ --type rust | grep -E "(1\.0|1\.1)" >> "$REPORT" || echo "No outdated TLS versions ✓" >> "$REPORT"

# Check input validation
echo -e "\n## 5. Input Validation" >> "$REPORT"
echo "### SQL Injection Prevention" >> "$REPORT"
rg "format!.*SELECT|format!.*INSERT|format!.*UPDATE" src/ --type rust >> "$REPORT" || echo "No dynamic SQL found ✓" >> "$REPORT"

echo "### Path Traversal Prevention" >> "$REPORT"
rg "\.\./" src/ --type rust >> "$REPORT" || echo "No path traversal patterns ✓" >> "$REPORT"

# Network exposure
echo -e "\n## 6. Network Exposure" >> "$REPORT"
echo "### Binding to 0.0.0.0" >> "$REPORT"
rg "0\.0\.0\.0|bind.*all" src/ --type rust -B 2 -A 2 >> "$REPORT"

# Authentication checks
echo -e "\n## 7. Authentication & Authorization" >> "$REPORT"
echo "### Missing auth checks" >> "$REPORT"
# Look for public endpoints without auth
rg "pub.*async.*fn.*handle|pub.*fn.*route" src/ --type rust -A 5 | grep -v "auth\|Auth\|token\|Token" >> "$REPORT" || echo "All endpoints appear protected ✓" >> "$REPORT"

# Cryptography
echo -e "\n## 8. Cryptography" >> "$REPORT"
echo "### Weak algorithms" >> "$REPORT"
rg -i "md5|sha1|des|rc4" src/ --type rust >> "$REPORT" || echo "No weak algorithms found ✓" >> "$REPORT"

# Summary
echo -e "\n## Summary" >> "$REPORT"
ISSUES=$(grep -c "✓" "$REPORT" || true)
echo "- Total checks passed: $ISSUES" >> "$REPORT"

echo "✅ Security review complete. Report saved to: $REPORT"

# Show critical issues
echo ""
echo "Critical issues to address:"
grep -A 2 "password\|secret\|0\.0\.0\.0" "$REPORT" | head -20 || echo "None found!"
