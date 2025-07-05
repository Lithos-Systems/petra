#!/bin/bash
# Simple benchmark runner with preset configurations

set -e

# Predefined configurations
declare -A CONFIGS
CONFIGS[quick]="SIGNALS=100,500 BLOCKS=10,50 FEATURES=--no-default-features"
CONFIGS[standard]="SIGNALS=100,1000,5000 BLOCKS=10,100,500 FEATURES=--features optimized"
CONFIGS[stress]="SIGNALS=1000,10000,50000 BLOCKS=100,1000,5000 FEATURES=--features enhanced-monitoring,optimized"
CONFIGS[memory]="SIGNALS=10000,50000,100000 BLOCKS=1000,5000,10000 FEATURES=--features optimized,realtime"
CONFIGS[edge]="SIGNALS=50,100,500 BLOCKS=5,10,50 FEATURES=--no-default-features"

print_usage() {
    cat << EOF
Usage: $0 [PRESET|OPTIONS]

Presets:
    quick       Fast test (< 30s): 100-500 signals, 10-50 blocks
    standard    Balanced test (2-3m): 100-5k signals, 10-500 blocks  
    stress      Heavy test (10+ m): 1k-50k signals, 100-5k blocks
    memory      Memory test: 10k-100k signals, 1k-10k blocks
    edge        Edge device: 50-500 signals, 5-50 blocks

Custom Options:
    --signals COUNT1,COUNT2,...    Custom signal counts
    --blocks COUNT1,COUNT2,...     Custom block counts  
    --features FLAGS               Custom feature flags
    --baseline NAME                Save baseline
    --compare NAME                 Compare with baseline

Examples:
    $0 quick                       # Run quick preset
    $0 --signals 1000 --blocks 100 # Custom single test
    $0 stress --baseline v1.0      # Stress test with baseline
    $0 --compare v1.0              # Compare with saved baseline

EOF
}

PRESET=""
CUSTOM_SIGNALS=""
CUSTOM_BLOCKS=""
CUSTOM_FEATURES=""
BASELINE=""
COMPARE=""

if [[ $# -eq 0 ]]; then
    print_usage
    exit 1
fi

if [[ -n "${CONFIGS[$1]}" ]]; then
    PRESET="$1"
    shift
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --signals)
            CUSTOM_SIGNALS="$2"
            shift 2
            ;;
        --blocks)
            CUSTOM_BLOCKS="$2"
            shift 2
            ;;
        --features)
            CUSTOM_FEATURES="$2"
            shift 2
            ;;
        --baseline)
            BASELINE="$2"
            shift 2
            ;;
        --compare)
            COMPARE="$2"
            shift 2
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            if [[ -n "${CONFIGS[$1]}" ]]; then
                PRESET="$1"
                shift
            else
                echo "Unknown option: $1"
                print_usage
                exit 1
            fi
            ;;
    esac
done

if [[ -n "$PRESET" ]]; then
    echo "🎯 Using preset: $PRESET"
    eval "${CONFIGS[$PRESET]}"
else
    echo "🔧 Using custom configuration"
    SIGNALS="$CUSTOM_SIGNALS"
    BLOCKS="$CUSTOM_BLOCKS"
    FEATURES="$CUSTOM_FEATURES"
fi

[[ -n "$CUSTOM_SIGNALS" ]] && SIGNALS="$CUSTOM_SIGNALS"
[[ -n "$CUSTOM_BLOCKS" ]] && BLOCKS="$CUSTOM_BLOCKS"
[[ -n "$CUSTOM_FEATURES" ]] && FEATURES="$CUSTOM_FEATURES"

if [[ -z "$SIGNALS" || -z "$BLOCKS" ]]; then
    echo "❌ Error: Signals and blocks must be specified"
    print_usage
    exit 1
fi

export PETRA_BENCH_SIGNALS="$SIGNALS"
export PETRA_BENCH_BLOCKS="$BLOCKS"

CMD="cargo bench $FEATURES --bench engine_performance"

if [[ -n "$BASELINE" ]]; then
    CMD="$CMD -- --save-baseline $BASELINE"
elif [[ -n "$COMPARE" ]]; then
    CMD="$CMD -- --baseline $COMPARE"
fi

echo "📊 Benchmark Configuration"
echo "=========================="
echo "Signals: $SIGNALS"
echo "Blocks: $BLOCKS"
echo "Features: ${FEATURES:-"(default)"}"
[[ -n "$BASELINE" ]] && echo "Baseline: $BASELINE"
[[ -n "$COMPARE" ]] && echo "Compare: $COMPARE"
echo ""

IFS=',' read -ra SIGNAL_ARRAY <<< "$SIGNALS"
count=${#SIGNAL_ARRAY[@]}
case $count in
    1|2) echo "⏱️  Estimated runtime: < 1 minute" ;;
    3|4) echo "⏱️  Estimated runtime: 2-3 minutes" ;;
    *) echo "⏱️  Estimated runtime: 5+ minutes" ;;
esac

if [[ ${SIGNALS} == *"50000"* ]] || [[ ${SIGNALS} == *"100000"* ]]; then
    echo "⚠️  Warning: This will run a stress test with large signal counts"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled"
        exit 0
    fi
fi

echo "🚀 Running: $CMD"
echo ""
eval $CMD

echo ""
echo "✅ Benchmark complete!"
echo "📊 View detailed reports: target/criterion/report/index.html"
