# Android Emulator VNC Setup Task

## Context

```yaml
context:
  environment:
    - setup: "Host → VM → Docker devcontainer"
    - constraint: "No host display hijacking"
    - goal: "Browser-based VNC access to Android emulator"
    
  current_issues:
    - issue: "X11 forwarding complexity in nested virtualization"
      fix: "Use headless Xvfb + VNC server"
    - issue: "No KVM in nested container"
      fix: "Use software rendering with swiftshader"
    - issue: "Port forwarding through multiple layers"
      fix: "Expose noVNC web port directly"
      
  gotchas:
    - issue: "Android emulator requires display even in headless mode"
      fix: "Use Xvfb virtual framebuffer"
    - issue: "noVNC needs websocket proxy"
      fix: "Include websockify in setup"
    - issue: "Emulator GPU acceleration fails without KVM"
      fix: "Force software rendering with -gpu swiftshader"
```

## Task Sequence

### 1. Clean Docker and DevContainer Setup

**ACTION** `.devcontainer/Dockerfile`:
  - OPERATION: Refactor to focus on VNC/noVNC setup
  - CHANGES:
    - Remove X11 forwarding dependencies
    - Add comprehensive VNC server setup (TigerVNC)
    - Add noVNC web client
    - Configure supervisor for service management
    - Set up proper user permissions
  - VALIDATE: `docker build -t test-vnc .devcontainer/`
  - IF_FAIL: Check package availability, use alternative VNC server
  - ROLLBACK: Keep original Dockerfile as backup

**ACTION** `.devcontainer/devcontainer.json`:
  - OPERATION: Update for VNC-based workflow
  - CHANGES:
    - Forward noVNC port (6080)
    - Remove X11 display references
    - Add startup command for VNC services
    - Configure proper environment variables
  - VALIDATE: Rebuild container and check port forwarding
  - IF_FAIL: Check port conflicts, adjust port numbers
  - ROLLBACK: Restore original configuration

### 2. VNC Service Configuration

**ACTION** `scripts/vnc/start-services.sh`:
  - OPERATION: Create comprehensive VNC startup script
  - FEATURES:
    - Start Xvfb on display :99
    - Launch VNC server (TigerVNC or x11vnc)
    - Start noVNC web server
    - Configure window manager (fluxbox/openbox)
    - Health check for all services
  - VALIDATE: Run script and check service status
  - IF_FAIL: Check logs in /var/log/vnc/
  - ROLLBACK: Kill all VNC processes

**ACTION** `scripts/vnc/supervisor-vnc.conf`:
  - OPERATION: Create supervisor configuration
  - SERVICES:
    - xvfb: Virtual display
    - vnc: VNC server
    - novnc: Web client
    - window-manager: Minimal WM
  - VALIDATE: `supervisorctl status`
  - IF_FAIL: Check individual service logs
  - ROLLBACK: Stop supervisor

### 3. Android Emulator Configuration

**ACTION** `scripts/emulator/create-avd.sh`:
  - OPERATION: Create AVD with software rendering
  - SETTINGS:
    - Use x86_64 image for performance
    - Configure for headless operation
    - Set memory and storage limits
    - Disable hardware acceleration
  - VALIDATE: `avdmanager list avd`
  - IF_FAIL: Delete AVD and recreate
  - ROLLBACK: Remove AVD directory

**ACTION** `scripts/emulator/start-headless.sh`:
  - OPERATION: Launch emulator in VNC display
  - FLAGS:
    - `-gpu swiftshader` (software rendering)
    - `-no-audio` (disable audio)
    - `-no-boot-anim` (faster startup)
    - `-memory 2048` (limit memory)
    - `-no-snapshot-save` (faster shutdown)
  - VALIDATE: Check emulator process and ADB connection
  - IF_FAIL: Try with `-gpu off` or `-gpu guest`
  - ROLLBACK: Kill emulator process

### 4. Web Interface Setup

**ACTION** `scripts/vnc/novnc-config.html`:
  - OPERATION: Create custom noVNC interface
  - FEATURES:
    - Auto-connect to VNC
    - Fullscreen support
    - Touch input mapping
    - Custom toolbar
  - VALIDATE: Access http://localhost:6080
  - IF_FAIL: Check websocket connection
  - ROLLBACK: Use default noVNC interface

**ACTION** `scripts/vnc/nginx.conf`:
  - OPERATION: Optional nginx proxy configuration
  - FEATURES:
    - WebSocket proxying
    - SSL termination (optional)
    - Path-based routing
  - VALIDATE: Test with curl/browser
  - IF_FAIL: Check nginx logs
  - ROLLBACK: Direct noVNC access

### 5. Integration Scripts

**ACTION** `scripts/dev-env.sh`:
  - OPERATION: Master control script
  - COMMANDS:
    - `start`: Launch all services
    - `stop`: Clean shutdown
    - `status`: Check all components
    - `logs`: Tail all logs
    - `emulator`: Emulator controls
  - VALIDATE: Run all commands
  - IF_FAIL: Check individual components
  - ROLLBACK: Manual service control

**ACTION** `scripts/test-connection.sh`:
  - OPERATION: Verify full stack
  - CHECKS:
    - VNC server running
    - noVNC accessible
    - Emulator responsive
    - ADB connected
    - AnkiDroid installable
  - VALIDATE: All checks pass
  - IF_FAIL: Debug specific failure
  - ROLLBACK: N/A (diagnostic only)

### 6. Documentation and Testing

**ACTION** `docs/VNC_SETUP.md`:
  - OPERATION: Create setup documentation
  - SECTIONS:
    - Architecture overview
    - Port mappings
    - Troubleshooting guide
    - Performance tuning
  - VALIDATE: Follow steps in clean environment
  - IF_FAIL: Update unclear sections
  - ROLLBACK: N/A

**ACTION** `.github/workflows/test-vnc.yml`:
  - OPERATION: Add CI test for VNC setup
  - TESTS:
    - Build Docker image
    - Start services
    - Connect to emulator
    - Run basic test
  - VALIDATE: Push and check CI
  - IF_FAIL: Adjust for CI environment
  - ROLLBACK: Disable workflow

## Validation Strategy

### Unit Tests
- Each script executable and syntax-valid
- Docker image builds successfully
- Ports are available and forwarded

### Integration Tests
1. Start VNC services → verify display created
2. Launch emulator → verify ADB connection
3. Access noVNC → verify visual output
4. Install AnkiDroid → verify app runs
5. Run demo app → verify plugin works

### Performance Checks
- Emulator boot time < 60s
- VNC latency < 100ms
- Memory usage < 4GB
- CPU usage reasonable

## Debug Strategies

### VNC Connection Issues
```bash
# Check VNC server
netstat -tlnp | grep 590
ps aux | grep vnc

# Test direct VNC
vncviewer localhost:5901

# Check Xvfb
DISPLAY=:99 xdpyinfo
```

### Emulator Issues
```bash
# Check emulator logs
cat ~/.android/avd/*.avd/config.ini
emulator -verbose -debug-all @avd_name

# ADB connection
adb devices
adb shell getprop
```

### noVNC Issues
```bash
# Check websocket
curl http://localhost:6080/
tail -f /var/log/novnc.log

# Browser console
# Check WebSocket connection in Network tab
```

## Rollback Plan

1. Stop all services: `supervisorctl stop all`
2. Kill remaining processes: `pkill -f vnc; pkill -f emulator`
3. Restore original Dockerfile/devcontainer.json
4. Rebuild container
5. Use fallback X11 forwarding if needed

## Success Criteria

- [ ] VNC accessible at http://localhost:6080
- [ ] Android emulator visible in browser
- [ ] No X11/display requirements on host
- [ ] AnkiDroid installable and functional
- [ ] Demo app can connect to AnkiDroid
- [ ] Works in nested virtualization
- [ ] < 60s to full environment ready

## Notes

- Prefer TigerVNC over x11vnc for stability
- Use supervisor for service management
- Keep emulator memory limited (2GB max)
- Consider adding audio support later if needed
- Can add SSL/auth to noVNC for security