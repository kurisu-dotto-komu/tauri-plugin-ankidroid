#!/bin/bash
set -e

# VNC Service Manager - Comprehensive startup script for VNC services
# This script manages Xvfb, VNC server, noVNC, and window manager

# Configuration
DISPLAY_NUM="${DISPLAY_NUM:-99}"
VNC_PORT="${VNC_PORT:-5901}"
NOVNC_PORT="${NOVNC_PORT:-6080}"
VNC_RESOLUTION="${VNC_RESOLUTION:-1280x1024}"
VNC_DEPTH="${VNC_DEPTH:-24}"
VNC_PASSWORD="${VNC_PASSWORD:-android}"
LOG_DIR="/var/log/vnc"

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

check_service() {
    local service=$1
    if pgrep -x "$service" > /dev/null; then
        return 0
    else
        return 1
    fi
}

cleanup_old_services() {
    log_info "Cleaning up old VNC services..."
    
    # Kill existing services
    pkill -f "Xvfb :${DISPLAY_NUM}" 2>/dev/null || true
    pkill -f "x11vnc.*:${DISPLAY_NUM}" 2>/dev/null || true
    pkill -f "novnc" 2>/dev/null || true
    pkill -f "websockify" 2>/dev/null || true
    pkill -f "fluxbox" 2>/dev/null || true
    
    # Clean up lock files
    rm -f /tmp/.X${DISPLAY_NUM}-lock 2>/dev/null || true
    rm -f /tmp/.X11-unix/X${DISPLAY_NUM} 2>/dev/null || true
    
    sleep 2
}

start_xvfb() {
    log_info "Starting Xvfb virtual display on :${DISPLAY_NUM}..."
    
    # Create log directory with proper permissions
    sudo mkdir -p ${LOG_DIR}
    sudo chmod 777 ${LOG_DIR}
    
    # Start Xvfb
    Xvfb :${DISPLAY_NUM} \
        -screen 0 ${VNC_RESOLUTION}x${VNC_DEPTH} \
        -ac \
        +extension GLX \
        +render \
        -noreset &> ${LOG_DIR}/xvfb.log &
    
    XVFB_PID=$!
    
    # Wait for Xvfb to start
    local max_attempts=10
    local attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if [ -e "/tmp/.X11-unix/X${DISPLAY_NUM}" ]; then
            log_info "✓ Xvfb started successfully (PID: ${XVFB_PID})"
            export DISPLAY=:${DISPLAY_NUM}
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log_error "Failed to start Xvfb"
    return 1
}

start_window_manager() {
    log_info "Starting window manager (fluxbox)..."
    
    DISPLAY=:${DISPLAY_NUM} fluxbox &> ${LOG_DIR}/fluxbox.log &
    WM_PID=$!
    
    sleep 2
    
    if check_service "fluxbox"; then
        log_info "✓ Window manager started (PID: ${WM_PID})"
        return 0
    else
        log_warn "Window manager failed to start (non-critical)"
        return 0
    fi
}

start_vnc_server() {
    log_info "Starting VNC server on port ${VNC_PORT}..."
    
    # Create password file if needed (skip if vncpasswd not available)
    if command -v vncpasswd &> /dev/null; then
        if [ ! -f ~/.vnc/passwd ]; then
            mkdir -p ~/.vnc
            echo "$VNC_PASSWORD" | vncpasswd -f > ~/.vnc/passwd
            chmod 600 ~/.vnc/passwd
        fi
    else
        log_warn "vncpasswd not found, using plain password"
    fi
    
    # Start x11vnc
    DISPLAY=:${DISPLAY_NUM} x11vnc \
        -display :${DISPLAY_NUM} \
        -rfbport ${VNC_PORT} \
        -forever \
        -shared \
        -xkb \
        -noxrecord \
        -noxfixes \
        -noxdamage \
        -passwd ${VNC_PASSWORD} \
        -bg \
        -o ${LOG_DIR}/x11vnc.log
    
    sleep 2
    
    if netstat -tlnp 2>/dev/null | grep -q ":${VNC_PORT}"; then
        log_info "✓ VNC server started on port ${VNC_PORT}"
        return 0
    else
        log_error "Failed to start VNC server"
        return 1
    fi
}

start_novnc() {
    log_info "Starting noVNC web server on port ${NOVNC_PORT}..."
    
    # Check if novnc is available
    if [ ! -d "/usr/share/novnc" ]; then
        log_error "noVNC not found at /usr/share/novnc"
        return 1
    fi
    
    # Start noVNC with websockify
    websockify --web /usr/share/novnc \
        ${NOVNC_PORT} localhost:${VNC_PORT} &> ${LOG_DIR}/novnc.log &
    
    NOVNC_PID=$!
    
    # Wait for noVNC to start
    local max_attempts=10
    local attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if netstat -tlnp 2>/dev/null | grep -q ":${NOVNC_PORT}"; then
            log_info "✓ noVNC started on port ${NOVNC_PORT} (PID: ${NOVNC_PID})"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log_error "Failed to start noVNC"
    return 1
}

health_check() {
    log_info "Performing health check..."
    
    local all_healthy=true
    
    # Check Xvfb
    if [ -e "/tmp/.X11-unix/X${DISPLAY_NUM}" ]; then
        log_info "✓ Xvfb is running"
    else
        log_error "✗ Xvfb is not running"
        all_healthy=false
    fi
    
    # Check VNC server
    if netstat -tlnp 2>/dev/null | grep -q ":${VNC_PORT}"; then
        log_info "✓ VNC server is listening on port ${VNC_PORT}"
    else
        log_error "✗ VNC server is not listening"
        all_healthy=false
    fi
    
    # Check noVNC
    if netstat -tlnp 2>/dev/null | grep -q ":${NOVNC_PORT}"; then
        log_info "✓ noVNC is listening on port ${NOVNC_PORT}"
    else
        log_error "✗ noVNC is not listening"
        all_healthy=false
    fi
    
    # Test display
    if DISPLAY=:${DISPLAY_NUM} xdpyinfo &>/dev/null; then
        log_info "✓ Display :${DISPLAY_NUM} is accessible"
    else
        log_error "✗ Display :${DISPLAY_NUM} is not accessible"
        all_healthy=false
    fi
    
    if [ "$all_healthy" = true ]; then
        log_info "✓ All services are healthy"
        return 0
    else
        log_error "Some services are not healthy"
        return 1
    fi
}

# Main execution
main() {
    log_info "=== VNC Service Manager ==="
    log_info "Configuration:"
    log_info "  Display: :${DISPLAY_NUM}"
    log_info "  VNC Port: ${VNC_PORT}"
    log_info "  noVNC Port: ${NOVNC_PORT}"
    log_info "  Resolution: ${VNC_RESOLUTION}x${VNC_DEPTH}"
    echo ""
    
    # Cleanup old services
    cleanup_old_services
    
    # Start services
    if ! start_xvfb; then
        log_error "Failed to start Xvfb"
        exit 1
    fi
    
    if ! start_window_manager; then
        log_warn "Window manager failed (continuing anyway)"
    fi
    
    if ! start_vnc_server; then
        log_error "Failed to start VNC server"
        exit 1
    fi
    
    if ! start_novnc; then
        log_error "Failed to start noVNC"
        exit 1
    fi
    
    echo ""
    log_info "=== Services Started Successfully ==="
    log_info "VNC server: vnc://localhost:${VNC_PORT} (password: ${VNC_PASSWORD})"
    log_info "Web interface: http://localhost:${NOVNC_PORT}"
    log_info "Display: :${DISPLAY_NUM}"
    echo ""
    
    # Perform health check
    sleep 2
    health_check
    
    echo ""
    log_info "Logs available at: ${LOG_DIR}/"
    log_info "To stop services: pkill -f 'Xvfb|x11vnc|novnc|fluxbox'"
}

# Handle script arguments
case "${1:-}" in
    stop)
        log_info "Stopping VNC services..."
        cleanup_old_services
        log_info "✓ Services stopped"
        ;;
    status)
        health_check
        ;;
    restart)
        log_info "Restarting VNC services..."
        cleanup_old_services
        sleep 2
        main
        ;;
    *)
        main
        ;;
esac