#!/bin/bash
set -e

# Android Emulator Headless Startup Script
# Launches emulator in VNC display with software rendering

# Configuration
AVD_NAME="${AVD_NAME:-tauri-ankidroid-test}"
DISPLAY_NUM="${DISPLAY_NUM:-99}"
MEMORY_SIZE="${MEMORY_SIZE:-6144}"
PARTITION_SIZE="${PARTITION_SIZE:-2048}"
CPU_CORES="${CPU_CORES:-1}"
BOOT_TIMEOUT="${BOOT_TIMEOUT:-300}"
ADB_PORT="${ADB_PORT:-5555}"

# Emulator flags for headless operation
GPU_MODE="${GPU_MODE:-swiftshader}"  # swiftshader, off, or guest
NO_WINDOW="${NO_WINDOW:-false}"
NO_AUDIO="${NO_AUDIO:-true}"
NO_BOOT_ANIM="${NO_BOOT_ANIM:-true}"
NO_SNAPSHOT="${NO_SNAPSHOT:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

check_requirements() {
    log_info "Checking requirements..."
    
    # Check if emulator is available
    if ! command -v emulator &> /dev/null; then
        log_error "Android emulator not found in PATH"
        exit 1
    fi
    
    # Check if adb is available
    if ! command -v adb &> /dev/null; then
        log_error "adb not found in PATH"
        exit 1
    fi
    
    # Check if AVD exists
    if ! avdmanager list avd 2>/dev/null | grep -q "Name: ${AVD_NAME}"; then
        log_error "AVD '${AVD_NAME}' not found"
        log_info "Please run: ./scripts/emulator/create-avd.sh"
        exit 1
    fi
    
    # Check if display is available
    if [ ! -e "/tmp/.X11-unix/X${DISPLAY_NUM}" ]; then
        log_warn "Display :${DISPLAY_NUM} not found, starting VNC services..."
        if [ -f "/workspaces/tauri-plugin-ankidroid/scripts/vnc-start-services.sh" ]; then
            /workspaces/tauri-plugin-ankidroid/scripts/vnc-start-services.sh
        else
            log_error "VNC services script not found"
            exit 1
        fi
    fi
    
    log_info "✓ All requirements met"
}

cleanup_existing() {
    log_info "Cleaning up existing emulator processes..."
    
    # Kill existing emulator for this AVD
    pkill -f "emulator.*${AVD_NAME}" 2>/dev/null || true
    
    # Clean up ADB
    adb kill-server 2>/dev/null || true
    sleep 2
    
    # Start fresh ADB server
    adb start-server
    
    log_info "✓ Cleanup complete"
}

build_emulator_command() {
    local cmd="emulator -avd ${AVD_NAME}"
    
    # GPU configuration
    cmd="${cmd} -gpu ${GPU_MODE}"
    
    # Enable hardware acceleration if KVM is available
    if [ -w /dev/kvm ]; then
        cmd="${cmd} -accel on"
    else
        cmd="${cmd} -accel off"
    fi
    
    # Window configuration
    if [ "${NO_WINDOW}" = "true" ]; then
        cmd="${cmd} -no-window"
    fi
    
    # Audio configuration
    if [ "${NO_AUDIO}" = "true" ]; then
        cmd="${cmd} -no-audio"
    fi
    
    # Boot animation
    if [ "${NO_BOOT_ANIM}" = "true" ]; then
        cmd="${cmd} -no-boot-anim"
    fi
    
    # Snapshot configuration (enable for faster boots)
    if [ "${NO_SNAPSHOT}" = "true" ]; then
        cmd="${cmd} -no-snapshot-save"
        cmd="${cmd} -no-snapshot-load"
    fi
    
    # Memory and CPU configuration
    cmd="${cmd} -memory ${MEMORY_SIZE}"
    cmd="${cmd} -partition-size ${PARTITION_SIZE}"
    cmd="${cmd} -cores ${CPU_CORES}"
    
    # Network configuration
    cmd="${cmd} -netdelay none"
    cmd="${cmd} -netspeed full"
    
    # Port configuration
    cmd="${cmd} -port ${ADB_PORT}"
    
    # Additional optimizations based on Linaro QEMU performance analysis
    cmd="${cmd} -no-metrics"
    cmd="${cmd} -no-passive-gps"
    cmd="${cmd} -cache-size 512"
    
    # QEMU CPU and SMP optimizations
    if [[ "${AVD_NAME}" == *"arm"* ]]; then
        # ARM64 optimizations: fast pointer authentication
        cmd="${cmd} -qemu -cpu max,pauth-impdef=on -smp ${CPU_CORES}"
    else
        # x86_64 optimizations: max CPU features, minimal cores
        cmd="${cmd} -qemu -cpu max -smp ${CPU_CORES}"
    fi
    
    # Display configuration
    export DISPLAY=:${DISPLAY_NUM}
    
    echo "${cmd}"
}

start_emulator() {
    log_info "Starting Android emulator..."
    log_info "  AVD: ${AVD_NAME}"
    log_info "  Display: :${DISPLAY_NUM}"
    log_info "  GPU Mode: ${GPU_MODE}"
    log_info "  Memory: ${MEMORY_SIZE}MB"
    log_info "  CPU Cores: ${CPU_CORES}"
    
    local emulator_cmd=$(build_emulator_command)
    
    log_debug "Command: ${emulator_cmd}"
    
    # Start emulator in background
    DISPLAY=:${DISPLAY_NUM} ${emulator_cmd} &> /var/log/emulator.log &
    local emulator_pid=$!
    
    log_info "✓ Emulator process started (PID: ${emulator_pid})"
    
    # Save PID for later reference
    echo ${emulator_pid} > /tmp/emulator_${AVD_NAME}.pid
    
    return 0
}

wait_for_device() {
    log_info "Waiting for device to connect..."
    
    local timeout=120
    local elapsed=0
    
    while [ ${elapsed} -lt ${timeout} ]; do
        if adb devices | grep -q "emulator\|localhost"; then
            log_info "✓ Device connected"
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
        echo -n "."
    done
    
    echo ""
    log_error "Device failed to connect within ${timeout} seconds"
    return 1
}

wait_for_boot() {
    log_info "Waiting for device to boot (this may take 2-5 minutes)..."
    
    local elapsed=0
    local last_status=""
    
    while [ ${elapsed} -lt ${BOOT_TIMEOUT} ]; do
        # Check if boot completed
        local boot_status=$(adb shell getprop sys.boot_completed 2>/dev/null || echo "0")
        
        if [ "${boot_status}" = "1" ]; then
            echo ""
            log_info "✓ Device booted successfully!"
            return 0
        fi
        
        # Show progress
        local current_status=$(adb shell getprop init.svc.bootanim 2>/dev/null || echo "unknown")
        if [ "${current_status}" != "${last_status}" ]; then
            echo ""
            log_info "Boot animation: ${current_status}"
            last_status="${current_status}"
        fi
        
        sleep 5
        elapsed=$((elapsed + 5))
        
        # Show time elapsed
        if [ $((elapsed % 30)) -eq 0 ]; then
            echo ""
            log_info "Still booting... (${elapsed}s elapsed)"
        else
            echo -n "."
        fi
    done
    
    echo ""
    log_error "Device failed to boot within ${BOOT_TIMEOUT} seconds"
    return 1
}

configure_device() {
    log_info "Configuring device settings..."
    
    # Wait a bit for system to stabilize
    sleep 5
    
    # Disable animations for better performance
    adb shell settings put global window_animation_scale 0.0
    adb shell settings put global transition_animation_scale 0.0
    adb shell settings put global animator_duration_scale 0.0
    
    # Keep screen on
    adb shell svc power stayon true
    
    # Set up for development
    adb shell settings put global development_settings_enabled 1
    adb shell settings put global adb_enabled 1
    
    log_info "✓ Device configured"
}

show_device_info() {
    log_info "=== Device Information ==="
    
    echo -e "${GREEN}ADB Devices:${NC}"
    adb devices
    
    echo ""
    echo -e "${GREEN}Device Properties:${NC}"
    echo "  Android Version: $(adb shell getprop ro.build.version.release)"
    echo "  API Level: $(adb shell getprop ro.build.version.sdk)"
    echo "  Device Model: $(adb shell getprop ro.product.model)"
    echo "  CPU ABI: $(adb shell getprop ro.product.cpu.abi)"
    echo "  Screen Density: $(adb shell getprop ro.sf.lcd_density)"
    
    echo ""
    log_info "=== Emulator Ready ==="
    log_info "VNC access: http://localhost:6080"
    log_info "ADB port: ${ADB_PORT}"
    
    if [ -f "/tmp/emulator_${AVD_NAME}.pid" ]; then
        local pid=$(cat /tmp/emulator_${AVD_NAME}.pid)
        log_info "Emulator PID: ${pid}"
    fi
    
    echo ""
    log_info "To install AnkiDroid:"
    echo "  ./scripts/emu-install-ankidroid.sh"
    echo ""
    log_info "To stop the emulator:"
    echo "  adb emu kill"
    echo "  # or"
    echo "  pkill -f 'emulator.*${AVD_NAME}'"
}

monitor_emulator() {
    log_info "Monitoring emulator output (tail -f /var/log/emulator.log)..."
    log_info "Press Ctrl+C to stop monitoring (emulator will continue running)"
    echo ""
    tail -f /var/log/emulator.log
}

# Main execution
main() {
    log_info "=== Android Emulator Launcher (Headless) ==="
    echo ""
    
    check_requirements
    cleanup_existing
    
    if ! start_emulator; then
        log_error "Failed to start emulator"
        exit 1
    fi
    
    if ! wait_for_device; then
        log_error "Device connection failed"
        
        # Show emulator log for debugging
        log_error "Emulator log tail:"
        tail -20 /var/log/emulator.log
        exit 1
    fi
    
    if ! wait_for_boot; then
        log_error "Device boot failed"
        
        # Show emulator log for debugging
        log_error "Emulator log tail:"
        tail -20 /var/log/emulator.log
        exit 1
    fi
    
    configure_device
    show_device_info
    
    # Optional: monitor emulator output
    if [ "${1:-}" = "--monitor" ]; then
        monitor_emulator
    fi
}

# Handle script arguments
case "${1:-}" in
    stop)
        log_info "Stopping emulator..."
        adb emu kill 2>/dev/null || true
        pkill -f "emulator.*${AVD_NAME}" 2>/dev/null || true
        rm -f /tmp/emulator_${AVD_NAME}.pid
        log_info "✓ Emulator stopped"
        ;;
    status)
        if [ -f "/tmp/emulator_${AVD_NAME}.pid" ]; then
            local pid=$(cat /tmp/emulator_${AVD_NAME}.pid)
            if ps -p ${pid} > /dev/null 2>&1; then
                log_info "Emulator is running (PID: ${pid})"
                adb devices
            else
                log_warn "Emulator PID file exists but process not running"
            fi
        else
            log_info "Emulator is not running"
        fi
        ;;
    restart)
        $0 stop
        sleep 2
        main
        ;;
    --monitor)
        main --monitor
        ;;
    *)
        main
        ;;
esac