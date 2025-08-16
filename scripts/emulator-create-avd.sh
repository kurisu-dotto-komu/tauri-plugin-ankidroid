#!/bin/bash
set -e

# Android AVD Creation Script - Optimized for software rendering in containers
# Creates an x86_64 AVD configured for headless operation with VNC

# Configuration
AVD_NAME="${AVD_NAME:-tauri-ankidroid-test}"
SYSTEM_IMAGE="${SYSTEM_IMAGE:-system-images;android-34;google_apis;x86_64}"
DEVICE_ID="${DEVICE_ID:-pixel_6}"
AVD_PATH="${HOME}/.android/avd"
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-/opt/android-sdk}"

# Memory and storage configuration
MEMORY_SIZE="${MEMORY_SIZE:-2048}"
HEAP_SIZE="${HEAP_SIZE:-256}"
STORAGE_SIZE="${STORAGE_SIZE:-2048M}"
CACHE_SIZE="${CACHE_SIZE:-64M}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

check_requirements() {
    log_info "Checking requirements..."
    
    # Check if Android SDK is installed
    if [ ! -d "${ANDROID_SDK_ROOT}" ]; then
        log_error "Android SDK not found at ${ANDROID_SDK_ROOT}"
        exit 1
    fi
    
    # Check if avdmanager is available
    if ! command -v avdmanager &> /dev/null; then
        log_error "avdmanager not found in PATH"
        exit 1
    fi
    
    # Check if sdkmanager is available
    if ! command -v sdkmanager &> /dev/null; then
        log_error "sdkmanager not found in PATH"
        exit 1
    fi
    
    log_info "✓ All requirements met"
}

install_system_image() {
    log_info "Checking system image: ${SYSTEM_IMAGE}"
    
    # Check if system image is already installed
    if sdkmanager --list_installed 2>/dev/null | grep -q "${SYSTEM_IMAGE}"; then
        log_info "✓ System image already installed"
    else
        log_info "Installing system image..."
        yes | sdkmanager --install "${SYSTEM_IMAGE}" || {
            log_error "Failed to install system image"
            exit 1
        }
        log_info "✓ System image installed successfully"
    fi
}

delete_existing_avd() {
    if avdmanager list avd 2>/dev/null | grep -q "Name: ${AVD_NAME}"; then
        log_warn "AVD '${AVD_NAME}' already exists. Deleting..."
        avdmanager delete avd -n "${AVD_NAME}" || {
            log_error "Failed to delete existing AVD"
            # Try manual cleanup
            rm -rf "${AVD_PATH}/${AVD_NAME}.avd" 2>/dev/null || true
            rm -rf "${AVD_PATH}/${AVD_NAME}.ini" 2>/dev/null || true
        }
        log_info "✓ Existing AVD deleted"
    fi
}

create_avd() {
    log_info "Creating AVD: ${AVD_NAME}"
    log_info "  Device: ${DEVICE_ID}"
    log_info "  System Image: ${SYSTEM_IMAGE}"
    log_info "  Memory: ${MEMORY_SIZE}MB"
    log_info "  Storage: ${STORAGE_SIZE}"
    
    # Create AVD with specific settings
    echo "no" | avdmanager create avd \
        -n "${AVD_NAME}" \
        -k "${SYSTEM_IMAGE}" \
        -d "${DEVICE_ID}" \
        -c "${STORAGE_SIZE}" \
        -f || {
        log_error "Failed to create AVD"
        exit 1
    }
    
    log_info "✓ AVD created successfully"
}

configure_avd() {
    log_info "Configuring AVD for headless operation..."
    
    local config_file="${AVD_PATH}/${AVD_NAME}.avd/config.ini"
    
    if [ ! -f "${config_file}" ]; then
        log_error "AVD config file not found: ${config_file}"
        exit 1
    fi
    
    # Backup original config
    cp "${config_file}" "${config_file}.backup"
    
    # Update configuration for headless operation
    cat >> "${config_file}" << EOF

# Custom settings for headless operation
hw.ramSize=${MEMORY_SIZE}
vm.heapSize=${HEAP_SIZE}
hw.gpu.enabled=yes
hw.gpu.mode=swiftshader_indirect
hw.keyboard=yes
hw.accelerometer=no
hw.audioInput=no
hw.battery=yes
hw.camera.back=none
hw.camera.front=none
hw.dPad=no
hw.device.manufacturer=Google
hw.gps=no
hw.lcd.density=420
hw.mainKeys=no
hw.sdCard=yes
sdcard.size=${CACHE_SIZE}
hw.sensors.orientation=no
hw.sensors.proximity=no
hw.trackBall=no
PlayStore.enabled=false
fastboot.forceColdBoot=yes
runtime.network.speed=full
runtime.network.latency=none
EOF
    
    # Create a custom advancedFeatures.ini for better container support
    local features_file="${AVD_PATH}/${AVD_NAME}.avd/advancedFeatures.ini"
    cat > "${features_file}" << EOF
Vulkan = off
GLDirectMem = on
GLDMA = off
GrallocSync = on
EncryptUserData = off
IntelPerformanceMonitoringUnit = off
OffworldSurfaces = off
QuickbootFileBacked = off
RefCountPipe = on
VirtioGpuNativeSync = off
VirtualScene = off
EOF
    
    log_info "✓ AVD configured for headless operation"
}

verify_avd() {
    log_info "Verifying AVD creation..."
    
    if avdmanager list avd 2>/dev/null | grep -q "Name: ${AVD_NAME}"; then
        log_info "✓ AVD '${AVD_NAME}' is available"
        
        # Show AVD details
        echo ""
        log_info "AVD Details:"
        avdmanager list avd 2>/dev/null | grep -A 10 "Name: ${AVD_NAME}" | sed 's/^/  /'
        echo ""
        
        return 0
    else
        log_error "AVD '${AVD_NAME}' not found in AVD list"
        return 1
    fi
}

show_usage() {
    log_info "AVD created successfully!"
    echo ""
    log_info "To start the emulator, run:"
    echo "  ./scripts/emulator-start.sh"
    echo ""
    log_info "Or manually with:"
    echo "  emulator -avd ${AVD_NAME} -gpu swiftshader -no-audio -no-boot-anim"
    echo ""
    log_info "AVD location: ${AVD_PATH}/${AVD_NAME}.avd"
}

# Main execution
main() {
    log_info "=== Android AVD Creator ==="
    echo ""
    
    check_requirements
    install_system_image
    delete_existing_avd
    create_avd
    configure_avd
    
    if verify_avd; then
        show_usage
    else
        log_error "AVD creation failed"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    delete)
        log_info "Deleting AVD: ${AVD_NAME}"
        delete_existing_avd
        log_info "✓ AVD deleted"
        ;;
    info)
        verify_avd
        ;;
    *)
        main
        ;;
esac